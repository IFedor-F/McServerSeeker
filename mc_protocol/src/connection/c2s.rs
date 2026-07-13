use bytes::BytesMut;

pub trait ServerBoundPacket {
    const MC_NAME: &'static str;
    fn encode_payload(self, buf: &mut BytesMut, protocol: i32);
}

pub trait ServerBoundState: Sized {
    const STATE_NAME: &'static str;
    fn encode(self, buf: &mut BytesMut, protocol: i32);
    fn get_id<T: 'static>(protocol: i32) -> Option<i32>;
}
#[macro_export]
macro_rules! impl_serverbound_state {
    (
        state = $state_name:expr;
        enum $enum_name:ident;

        match protocol {
            $(
                $proto_pat:pat => {
                    $( $id:expr => $variant:ident : $packet_type:ident ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        impl ServerBoundState for $enum_name {
            const STATE_NAME: &'static str = $state_name;

            fn encode(self, buf: &mut bytes::BytesMut, protocol: i32) {
                match protocol {
                    $(
                        $proto_pat => {
                            match self {
                                $(
                                    Self::$variant(packet) => {
                                        crate::types::varint::McVarInt($id).write_to_buf(buf);
                                        packet.encode_payload(buf, protocol);
                                    }
                                )*
                                #[allow(unreachable_patterns)]
                                _ => panic!("packet isn't support in state {} with protocol {}", Self::STATE_NAME, protocol),
                            }
                        }
                    )*
                    _ => panic!("protocol {} doesn't support state {}", protocol, Self::STATE_NAME),
                }
            }

            fn get_id<T: 'static>(protocol: i32) -> Option<i32> {
                let target = std::any::TypeId::of::<T>();
                match protocol {
                    $(
                        $proto_pat => {
                            $(
                                if target == std::any::TypeId::of::<$packet_type>() {
                                    return Some($id);
                                }
                            )*
                            None
                        }
                    )*
                    _ => None
                }
            }
        }
    };
}
