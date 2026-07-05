use super::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError, try_split_to};
use crate::impl_clientbound_state;
use crate::mc_protocol::McPacket;
use crate::mc_protocol::types::mc_string::McStringField;
use crate::mc_protocol::types::varint::McVarInt;
use bytes::{Buf, Bytes};

#[derive(Debug)]
pub enum PacketEnum {
    Commands(CommandsPacket),
    Another(AnotherPacket),
}
impl ClientBoundState for PacketEnum {
    const STATE_NAME: &'static str = "Play";

    fn parse_packet(raw_packet: McPacket) -> Result<Self, PacketError> {
        let id = raw_packet.id;
        let payload = raw_packet.payload;
        match id {
            CommandsPacket::ID => Ok(Self::Commands(CommandsPacket::parse(payload).map_err(
                |e| PacketError::PacketDecoding {
                    state: Self::STATE_NAME,
                    packet_name: CommandsPacket::MC_NAME,
                    source: e,
                },
            )?)),
            0..=138 => Ok(Self::Another(AnotherPacket::parse(payload).map_err(
                |e| PacketError::PacketDecoding {
                    state: Self::STATE_NAME,
                    packet_name: AnotherPacket::MC_NAME,
                    source: e,
                },
            )?)),
            _ => Err(PacketError::InvalidPacketID {
                id,
                state: Self::STATE_NAME,
            }),
        }
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Commands

#[derive(Debug)]
pub struct CommandsPacket {
    pub command_names: Vec<String>,
}

impl ClientBoundPacket for CommandsPacket {
    const ID: i32 = 0x10;
    const MC_NAME: &str = "commands";

    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
        let mut command_names = Vec::new();

        for _ in 0..count {
            if data.is_empty() {
                break;
            }

            let flags = data.get_u8();
            let node_type = flags & 0x03;
            let children_count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;

            for _ in 0..children_count {
                McVarInt::read_from_buf(&mut data)?;
            }
            if (flags & 0x08) != 0 {
                McVarInt::read_from_buf(&mut data)?;
            }
            if node_type == 1 || node_type == 2 {
                let name = McStringField(32767).read_from_buf(&mut data)?;
                command_names.push(name);
            }
            if node_type == 2 {
                break;
            }
        }

        Ok(Self {
            command_names,
        })
    }
}

// ==========================================
// 2. Another (Fallback Packet)
// ==========================================

#[derive(Debug)]
pub struct AnotherPacket;

impl ClientBoundPacket for AnotherPacket {
    const ID: i32 = -1;
    const MC_NAME: &str = "unimplemented_packet";

    fn parse(_: Bytes) -> Result<Self, ParsePacketError> {
        Ok(Self)
    }
}
