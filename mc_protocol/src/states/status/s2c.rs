pub mod status_response;
pub use status_response::StatusResponsePacket;
pub mod pong_response;
pub use pong_response::PongResponsePacket;

use crate::connection::McPacket;
use crate::connection::s2c::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError};
use crate::impl_clientbound_state;

#[derive(Debug)]
pub enum S2CStatusState {
    StatusResponse(StatusResponsePacket),
    PongResponse(PongResponsePacket),
}

impl_clientbound_state! {
    state = "status";
    enum S2CStatusState;

    match protocol {
        0.. => {
            0x00 => StatusResponse : StatusResponsePacket,
            0x01 => PongResponse : PongResponsePacket,
        }
    }
}
