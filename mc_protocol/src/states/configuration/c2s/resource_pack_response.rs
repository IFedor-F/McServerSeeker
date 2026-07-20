use super::ServerBoundPacket;
use super::types::ResourcePackIdent;
use crate::types::{McVarInt, mc_string};
use bytes::{BufMut, BytesMut};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Resource_Pack_Response
#[derive(Debug)]
pub struct ResourcePackResponsePacket {
    pub ident: ResourcePackIdent,
    pub result: i32,
}
impl ResourcePackResponsePacket {
    pub fn successfully_downloaded(ident: ResourcePackIdent) -> Self {
        Self { ident, result: 0 } // 0 is 'successfully_downloaded'
    }
    pub fn accepted(ident: ResourcePackIdent) -> Self {
        Self { ident, result: 3 } // 3 is 'accepted'
    }
}
impl ServerBoundPacket for ResourcePackResponsePacket {
    const MC_NAME: &'static str = "resource_pack";
    fn encode_payload(self, buf: &mut BytesMut, protocol: i32) {
        match protocol {
            ..=209 => {
                if let ResourcePackIdent::Hash(hash) = self.ident {
                    mc_string::write_to_buf(&hash, buf);
                } else {
                    panic!(
                        "expected ident as hash in protocol {}, got {:?}",
                        protocol, self.ident
                    )
                }
            }
            210..=764 => {}
            765.. => {
                if let ResourcePackIdent::UUID(uuid) = self.ident {
                    buf.put_u128(uuid.as_u128());
                } else {
                    panic!(
                        "expected ident as uuid in protocol {}, got {:?}",
                        protocol, self.ident
                    )
                }
            }
        }
        McVarInt(self.result).write_to_buf(buf);
    }
}
