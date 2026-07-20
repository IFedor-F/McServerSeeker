use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McStringField, McVarInt};
use bytes::{Buf, Bytes};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Commands
#[derive(Debug)]
pub struct CommandsPacket {
    pub command_names: Vec<String>,
}

impl ClientBoundPacket for CommandsPacket {
    const MC_NAME: &str = "commands";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
        let mut command_names = Vec::new();

        for _ in 0..count {
            if data.is_empty() {
                break;
            }

            let flags = data.try_get_u8()?;
            let node_type = flags & 0x03;
            let children_count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;

            for _ in 0..children_count {
                McVarInt::read_from_buf(&mut data)?;
            }
            if (flags & 0x08) != 0 {
                McVarInt::read_from_buf(&mut data)?;
            }
            if node_type == 1 || node_type == 2 {
                let name = McStringField::<32767>::read_from_buf(&mut data)?;
                command_names.push(name);
            }
            if node_type == 2 {
                break;
            }
        }

        Ok(Self { command_names })
    }
}
