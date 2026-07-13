use super::ServerBoundPacket;
use crate::types::{McBool, Player, mc_string};
use bytes::BytesMut;
use uuid::Uuid;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets
// Currently supports only offline mode
#[derive(Debug)]
pub struct HelloPacket {
    pub name: String,
    pub uuid: Option<Uuid>,
}

impl HelloPacket {
    pub fn from_player(player: Player) -> Self {
        Self {
            name: player.name,
            uuid: Some(player.uuid),
        }
    }
}
impl ServerBoundPacket for HelloPacket {
    const MC_NAME: &'static str = "hello";
    fn encode_payload(self, buf: &mut BytesMut, protocol: i32) {
        mc_string::write_to_buf(&self.name, buf);
        match protocol {
            ..=758 => {}
            759 => {
                McBool(false).write_to_buf(buf);
            }
            760 => {
                McBool(false).write_to_buf(buf);
                match self.uuid {
                    None => {
                        McBool(false).write_to_buf(buf);
                    }
                    Some(uuid) => {
                        McBool(true).write_to_buf(buf);
                        buf.extend_from_slice(uuid.as_bytes());
                    }
                }
            }
            761..=763 => match self.uuid {
                None => {
                    McBool(false).write_to_buf(buf);
                }
                Some(uuid) => {
                    McBool(true).write_to_buf(buf);
                    buf.extend_from_slice(uuid.as_bytes());
                }
            },
            764.. => {
                let Some(uuid) = self.uuid else {
                    panic!(
                        "Packet {} expected uuid since protocol >= 764, actual: {}",
                        Self::MC_NAME,
                        protocol
                    );
                };
                buf.extend_from_slice(uuid.as_bytes());
            }
        }
    }
}
