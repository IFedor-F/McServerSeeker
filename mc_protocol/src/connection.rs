use crate::connection::s2c::ClientBoundState;
use crate::types::{McReadBuf, McVarInt, McVarIntError};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use c2s::ServerBoundState;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use s2c::PacketError;
use std::io::{Read, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub mod c2s;
pub mod s2c;

#[derive(thiserror::Error, Debug)]
pub enum McConnectionError {
    #[error("Connection error: too Big Packet ({0} bytes)")]
    TooBigPacket(usize),

    #[error("Connection error: I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection error: invalid McVarInt: {0}")]
    InvalidVarint(McVarIntError),

    #[error("Connection error: unexpectedEof")]
    UnexpectedEof,

    #[error("Connection error: parse packet error: {0}")]
    ParsePacket(#[from] PacketError),

    #[error("Connection error: invalid length")]
    InvalidLength,

    #[error("Connection error: zlib decompress error")]
    DecompressionError(#[from] flate2::DecompressError),

    #[error(
        "Connection error: decompressed size mismatch (declared: {declared}, actual: {actual})"
    )]
    DecompressedSizeMismatch { declared: usize, actual: usize },
}
impl From<McVarIntError> for McConnectionError {
    fn from(err: McVarIntError) -> Self {
        match err {
            McVarIntError::Io(err) => Self::Io(err),
            err => McConnectionError::InvalidVarint(err),
        }
    }
}

#[derive(Debug)]
pub struct McPacket {
    pub id: i32,
    pub payload: Bytes,
}

#[derive(Debug)]
pub enum FilteredMcPacket {
    Matched(McPacket),
    Unmatched(i32),
}

pub struct McConnection {
    pub stream: TcpStream,
    pub protocol: i32,

    read_buf: BytesMut,
    write_buf: BytesMut,
    scratch_buf: BytesMut,
    compress_buf: BytesMut,
    compression_threshold: Option<usize>,
}

impl McConnection {
    // Max size of raw packet is 2 MiB;
    const MAX_PACKET_SIZE: i32 = 2097151; // 2 MiB

    // Minecraft doesn't have a client-side data limit, but the server can potentially send compression bomb
    const MAX_UNCOMPRESSED_PACKET_SIZE: i32 = 4194304; // 4 MiB
    pub fn new(stream: TcpStream, protocol: i32) -> Self {
        Self {
            stream,
            protocol,
            read_buf: BytesMut::with_capacity(32 * 1024), // 32 KiB
            write_buf: BytesMut::with_capacity(1024),     // 1 KiB
            scratch_buf: BytesMut::with_capacity(1024),   // 1 KiB
            compress_buf: BytesMut::with_capacity(0),
            compression_threshold: None,
        }
    }
    pub fn enable_compress(&mut self, threshold: usize) {
        if self.compression_threshold.is_none() {
            self.compress_buf.reserve(32 * 1024); // 32 KiB
        }
        self.compression_threshold = Some(threshold);
    }

    pub fn queue_packet<P: ServerBoundState>(&mut self, packet: P) {
        self.scratch_buf.clear();
        packet.encode(&mut self.scratch_buf, self.protocol);
        let uncompressed_len = self.scratch_buf.len();
        match self.compression_threshold {
            // With compression enabled and more than threshold
            Some(threshold) if uncompressed_len >= threshold => {
                let mut temp_buf = std::mem::take(&mut self.compress_buf);
                temp_buf.clear();

                let mut encoder = ZlibEncoder::new(temp_buf.writer(), Compression::default());
                encoder.write_all(&self.scratch_buf).unwrap();
                let writer = encoder.finish().unwrap();

                self.compress_buf = writer.into_inner();

                let data_length_size = McVarInt(uncompressed_len as i32).len();
                let packet_length = data_length_size + self.compress_buf.len();

                McVarInt(packet_length as i32).write_to_buf(&mut self.write_buf);
                McVarInt(uncompressed_len as i32).write_to_buf(&mut self.write_buf);
                self.write_buf.extend_from_slice(&self.compress_buf);
            }
            // With compression enabled and lower than threshold
            Some(_) => {
                let data_length_size = McVarInt(0).len();
                let packet_length = data_length_size + uncompressed_len;

                McVarInt(packet_length as i32).write_to_buf(&mut self.write_buf);
                McVarInt(0).write_to_buf(&mut self.write_buf);
                self.write_buf.extend_from_slice(&self.scratch_buf);
            }
            None => {
                McVarInt(uncompressed_len as i32).write_to_buf(&mut self.write_buf);
                self.write_buf.extend_from_slice(&self.scratch_buf);
            }
        }
    }

    pub async fn flush(&mut self) -> Result<(), McConnectionError> {
        if !self.write_buf.is_empty() {
            self.stream.write_all(&self.write_buf).await?;
            self.write_buf.clear();
        }
        Ok(())
    }
    /// Add packet to queue and then call flush
    pub async fn send_packet<P: ServerBoundState>(
        &mut self,
        packet: P,
    ) -> Result<(), McConnectionError> {
        self.queue_packet(packet);
        self.flush().await?;
        Ok(())
    }

    pub async fn read_packet<P: ClientBoundState>(&mut self) -> Result<P, McConnectionError> {
        let length = self
            .read_varint()
            .await?
            .with_check(0, Self::MAX_PACKET_SIZE)?
            .0 as usize;

        match self.compression_threshold {
            // 1. Compression is enabled with threshold
            Some(_) => {
                let uncompressed_len_varint = self
                    .read_varint()
                    .await?
                    .with_check(0, Self::MAX_UNCOMPRESSED_PACKET_SIZE)?;

                let uncompressed_len = uncompressed_len_varint.0 as usize;
                let remaining_packet_len = length
                    .checked_sub(uncompressed_len_varint.len())
                    .ok_or(McConnectionError::InvalidLength)?;

                // 1.1 Compression enabled, but data is uncompressed (probably payload len is less than threshold)
                if uncompressed_len == 0 {
                    let mut packet_data = self.read_exact_bytes(remaining_packet_len).await?;
                    let id = McVarInt::read_from_buf(&mut packet_data)?.0;
                    let packet = McPacket {
                        id,
                        payload: packet_data,
                    };
                    Ok(P::parse_packet(packet, self.protocol)?)
                }
                // 1.2 Compression enabled and data is compressed (probable payload len is more than threshold)
                else {
                    let compressed_bytes = self.read_exact_bytes(remaining_packet_len).await?;
                    let mut decoder =
                        ZlibDecoder::new(&compressed_bytes[..]).take(uncompressed_len as u64);

                    let mut decompressed_data = Vec::with_capacity(uncompressed_len);
                    decoder.read_to_end(&mut decompressed_data)?;

                    if decompressed_data.len() != uncompressed_len {
                        return Err(McConnectionError::DecompressedSizeMismatch {
                            declared: uncompressed_len,
                            actual: decompressed_data.len(),
                        });
                    }

                    let mut decompressed_bytes = Bytes::from(decompressed_data);
                    let id = McVarInt::read_from_buf(&mut decompressed_bytes)?.0;
                    let packet = McPacket {
                        id,
                        payload: decompressed_bytes,
                    };
                    Ok(P::parse_packet(packet, self.protocol)?)
                }
            }
            // 2. Compression is disabled
            None => {
                let mut packet_data = self.read_exact_bytes(length).await?;
                let id = McVarInt::read_from_buf(&mut packet_data)?.0;
                let packet = McPacket {
                    id,
                    payload: packet_data,
                };
                Ok(P::parse_packet(packet, self.protocol)?)
            }
        }
    }
    pub async fn read_filtered_raw<F>(
        &mut self,
        filter: F,
    ) -> Result<FilteredMcPacket, McConnectionError>
    where
        F: Fn(i32) -> bool,
    {
        let length = self
            .read_varint()
            .await?
            .with_check(0, Self::MAX_PACKET_SIZE)?
            .0 as usize;

        match self.compression_threshold {
            // 1. Compression is enabled with threshold
            Some(_) => {
                let uncompressed_len_varint = self
                    .read_varint()
                    .await?
                    .with_check(0, Self::MAX_UNCOMPRESSED_PACKET_SIZE)?;

                let uncompressed_len = uncompressed_len_varint.0 as usize;
                let remaining_packet_len = length
                    .checked_sub(uncompressed_len_varint.len())
                    .ok_or(McConnectionError::InvalidLength)?; // Len of compressed id + payload

                // 1.1 Compression enabled, but data is uncompressed (probably payload len is less than threshold)
                if uncompressed_len == 0 {
                    let id_varint = self.read_varint().await?;
                    let payload_len = remaining_packet_len
                        .checked_sub(id_varint.len())
                        .ok_or(McConnectionError::InvalidLength)?;
                    match filter(id_varint.0) {
                        true => {
                            let payload = self.read_exact_bytes(payload_len).await?;

                            Ok(FilteredMcPacket::Matched(McPacket {
                                id: id_varint.0,
                                payload,
                            }))
                        }
                        false => {
                            self.skip_bytes(payload_len).await?;
                            Ok(FilteredMcPacket::Unmatched(id_varint.0))
                        }
                    }
                }
                // 1.2 Compression enabled and data is compressed (probable payload len is more than threshold)
                else {
                    let compressed_bytes = self.read_exact_bytes(remaining_packet_len).await?;
                    let mut decoder =
                        ZlibDecoder::new(&compressed_bytes[..]).take(uncompressed_len as u64);

                    let id_varint = McVarInt::read_from_reader(&mut decoder)?;
                    match filter(id_varint.0) {
                        true => {
                            let mut decompressed_payload = Vec::with_capacity(uncompressed_len);
                            decoder.read_to_end(&mut decompressed_payload)?;
                            let decompressed_payload = Bytes::from(decompressed_payload);
                            Ok(FilteredMcPacket::Matched(McPacket {
                                id: id_varint.0,
                                payload: decompressed_payload,
                            }))
                        }
                        false => Ok(FilteredMcPacket::Unmatched(id_varint.0)),
                    }
                }
            }
            // 2. Compression is disabled
            None => {
                let id_varint = self.read_varint().await?;
                let payload_len = length
                    .checked_sub(id_varint.len())
                    .ok_or(McConnectionError::InvalidLength)?;
                let payload = self.read_exact_bytes(payload_len).await?;
                match filter(id_varint.0) {
                    true => Ok(FilteredMcPacket::Matched(McPacket {
                        id: id_varint.0,
                        payload,
                    })),
                    false => {
                        self.skip_bytes(payload_len).await?;
                        Ok(FilteredMcPacket::Unmatched(id_varint.0))
                    }
                }
            }
        }
    }

    pub async fn read_filtered_packet<F, P>(&mut self, filter: F) -> Result<P, McConnectionError>
    where
        F: (Fn(i32) -> bool),
        P: ClientBoundState,
    {
        loop {
            let packet = self.read_filtered_raw(&filter).await?;
            match packet {
                FilteredMcPacket::Matched(p) => {
                    return Ok(P::parse_packet(p, self.protocol)?);
                }
                FilteredMcPacket::Unmatched(_) => {}
            };
        }
    }
    async fn read_exact_bytes(&mut self, len: usize) -> Result<Bytes, McConnectionError> {
        while self.read_buf.len() < len {
            let read = self.stream.read_buf(&mut self.read_buf).await?;
            if read == 0 {
                return Err(McConnectionError::UnexpectedEof);
            }
        }
        Ok(self.read_buf.split_to(len).freeze())
    }

    async fn skip_bytes(&mut self, mut len: usize) -> Result<(), McConnectionError> {
        let in_buffer = self.read_buf.len().min(len);
        self.read_buf.advance(in_buffer);
        len -= in_buffer;

        while len > 0 {
            let mut temp = BytesMut::with_capacity(len.min(8192));
            let read = self.stream.read_buf(&mut temp).await?;
            if read == 0 {
                return Err(McConnectionError::UnexpectedEof);
            }
            len -= read.min(len);
        }
        Ok(())
    }
    async fn read_varint(&mut self) -> Result<McVarInt, McConnectionError> {
        let mut num_read = 0;
        let mut result = 0;

        loop {
            if self.read_buf.is_empty() {
                let read = self.stream.read_buf(&mut self.read_buf).await?;
                if read == 0 {
                    return Err(McConnectionError::UnexpectedEof);
                }
            }

            let byte = self.read_buf[0];
            self.read_buf.advance(1);

            let value = (byte & 0b0111_1111) as i32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err(McConnectionError::InvalidVarint(McVarIntError::TooLong));
            }

            if (byte & 0b1000_0000) == 0 {
                break;
            }
        }
        Ok(McVarInt(result))
    }
}
