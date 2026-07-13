use super::McPacket;
use crate::types::{
    MCJsonFieldError, McBoolError, McNbtFieldError, McPrefixedArrayError, McStringFieldError,
    McVarIntError,
};
use bytes::{Buf, Bytes};

#[derive(Debug, thiserror::Error)]
pub enum ParsePacketError {
    #[error("McVarInt error: {0}")]
    InvalidVarInt(#[from] McVarIntError),

    #[error("McStringField error: {0}")]
    InvalidString(#[from] McStringFieldError),

    #[error("McBool error: {0}")]
    InvalidBool(#[from] McBoolError),

    #[error("McPrefixedArray error: {0}")]
    InvalidPrefixedArray(#[from] McPrefixedArrayError),

    #[error("McNbtField error: {0}")]
    InvalidNbt(#[from] McNbtFieldError),

    #[error("JSON text field parse error: {0}")]
    InvalidJsonTextField(#[from] MCJsonFieldError),

    #[error("Invalid UUID-string")]
    InvalidStringUuid,

    #[error("Length can't be less than 0")]
    NegativeLength,

    #[error("JSON parse error")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Field is too big (max expected: {max_expected}, actual: {actual})")]
    TooLongField { max_expected: usize, actual: usize },

    #[error("Unexpected EOF")]
    UnexpectedEof,
}
impl From<bytes::TryGetError> for ParsePacketError {
    fn from(_: bytes::TryGetError) -> Self {
        Self::UnexpectedEof
    }
}
#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("Invalid packet id: {id} in state: {state}")]
    InvalidPacketID { state: &'static str, id: i32 },

    #[error("Error parsing packet '{packet_name}': {source}")]
    PacketDecoding {
        state: &'static str,
        packet_name: &'static str,
        source: ParsePacketError,
    },
}

pub trait ClientBoundState: Sized {
    const STATE_NAME: &'static str;

    fn parse_packet(packet: McPacket, protocol: i32) -> Result<Self, PacketError>;
    fn is_packet<T: 'static>(id: i32, protocol: i32) -> bool;
    fn packet_name(id: i32, protocol: i32) -> Option<&'static str>;
}

pub trait ClientBoundPacket: Sized {
    const MC_NAME: &str;
    fn parse(data: Bytes, protocol: i32) -> Result<Self, ParsePacketError>;
}
#[macro_export]
macro_rules! impl_clientbound_state {
    (
        state = $state_name:expr;
        enum $enum_name:ident;

        match protocol {
            $(
                $proto_pat:pat => {
                    $( $id:pat => $variant:ident : $packet_type:ident ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        impl ClientBoundState for $enum_name {
            const STATE_NAME: &'static str = $state_name;

            fn parse_packet(packet: McPacket, protocol: i32) -> Result<Self, PacketError> {
                let id = packet.id;
                let payload = packet.payload;
                match protocol {
                    $(
                        $proto_pat => {
                            match id {
                                $(
                                    $id => {
                                        Ok(Self::$variant(
                                            $packet_type::parse(payload, protocol).map_err(|e| {
                                                PacketError::PacketDecoding {
                                                    state: Self::STATE_NAME,
                                                    packet_name: $packet_type::MC_NAME,
                                                    source: e,
                                                }
                                            })?,
                                        ))
                                    }
                                )*
                                #[allow(unreachable_patterns)]
                                _ => Err(PacketError::InvalidPacketID { id, state: Self::STATE_NAME }),
                            }
                        }
                    )*
                    _ => panic!("protocol {} doesn't support state {}", protocol, Self::STATE_NAME),
                }
            }

            fn is_packet<T: 'static>(id: i32, protocol: i32) -> bool {
                let target = std::any::TypeId::of::<T>();
                match protocol {
                    $(
                        $proto_pat => {
                            match id {
                                $(
                                    $id => std::any::TypeId::of::<$packet_type>() == target,
                                )*
                                _ => false,
                            }
                        }
                    )*
                    _ => false
                }
            }

            fn packet_name(id: i32, protocol: i32) -> Option<&'static str> {
                match protocol {
                    $(
                        $proto_pat => {
                            match id {
                                $(
                                    $id => Some($packet_type::MC_NAME),
                                )*
                                _ => None,
                            }
                        }
                    )*
                    _ => None
                }
            }
        }
    };
}
#[macro_export]
macro_rules! is_packet_any {
    ($state:ty, $id:expr, $protocol:expr, [ $( $packet_type:ty ),* $(,)? ]) => {
        false $( || <$state>::is_packet::<$packet_type>($id, $protocol) )*
    };
}

pub fn try_split_to(data: &mut Bytes, len: usize) -> Result<Bytes, ParsePacketError> {
    if data.len() < len {
        Err(ParsePacketError::UnexpectedEof)
    } else {
        Ok(data.split_to(len))
    }
}

pub fn try_advance(data: &mut Bytes, cnt: usize) -> Result<(), ParsePacketError> {
    if data.len() < cnt {
        Err(ParsePacketError::UnexpectedEof)
    } else {
        data.advance(cnt);
        Ok(())
    }
}
