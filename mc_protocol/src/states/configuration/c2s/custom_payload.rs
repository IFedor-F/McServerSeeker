use super::ServerBoundPacket;
use crate::types::mc_string;
use bytes::{BufMut, Bytes, BytesMut};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(serverbound)
#[derive(Debug)]
pub struct CustomPayloadResponsePacket {
    pub channel: String,
    pub data: Bytes,
}

impl ServerBoundPacket for CustomPayloadResponsePacket {
    const MC_NAME: &'static str = "custom_payload";
    fn encode_payload(self, buf: &mut BytesMut, protocol: i32) {
        mc_string::write_to_buf(&self.channel, buf);
        match protocol {
            ..=46 => {
                // this versions use short int as length before data bytes
                buf.put_i16(self.data.len() as i16);
                buf.extend(self.data);
            }
            47.. => {
                // since 47 there is no length before data
                buf.extend(self.data);
            }
        }
    }
}
