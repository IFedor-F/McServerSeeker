use crate::mc_protocol::McPacket;
use crate::mc_protocol::types::{MCJsonFieldError, MCStringFieldError, McBoolError, McVarIntError};
use bytes::Bytes;

pub mod login;
pub mod status;
pub mod configuration;
pub mod play;

#[derive(Debug, thiserror::Error)]
pub enum ParsePacketError {
    #[error("VarInt error: {0}")]
    InvalidVarInt(#[from] McVarIntError),

    #[error("MCString error: {0}")]
    InvalidString(#[from] MCStringFieldError),

    #[error("McBool error: {0}")]
    InvalidBool(#[from] McBoolError),

    #[error("Length can't be less than 0")]
    NegativeLength,

    #[error("JSON text field parse error: {0}")]
    InvalidJsonTextField(#[from] MCJsonFieldError),

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
    fn parse_packet(packet: McPacket) -> Result<Self, PacketError>;
}

pub trait ClientBoundPacket: Sized {
    const ID: i32;
    const MC_NAME: &str;
    fn parse(data: bytes::Bytes) -> Result<Self, ParsePacketError>;
}
#[macro_export]
macro_rules! impl_clientbound_state {
    (
        $state_enum:ident, $state_name:expr,
        $($variant:ident => $packet_type:ident),+ $(,)?
    ) => {
        impl ClientBoundState for $state_enum {
            const STATE_NAME: &'static str = $state_name;

            fn parse_packet(raw_packet: McPacket) -> Result<Self, PacketError> {
                let id = raw_packet.id;
                let payload = raw_packet.payload;
                match id {
                    $(
                        $packet_type::ID => Ok(Self::$variant($packet_type::parse(payload).map_err(|e| PacketError::PacketDecoding{state: Self::STATE_NAME, packet_name: $packet_type::MC_NAME, source: e})?)),
                    )+
                    _ => Err(PacketError::InvalidPacketID {
                        id,
                        state: Self::STATE_NAME,
                    }),
                }
            }
        }
    };
}

fn try_split_to(data: &mut Bytes, len: usize) -> Result<Bytes, ParsePacketError> {
    if data.len() < len {
        Err(ParsePacketError::UnexpectedEof)
    } else {
        Ok(data.split_to(len))
    }
}
