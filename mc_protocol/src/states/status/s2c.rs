mod status_response;
pub use status_response::StatusResponsePacket;
mod pong_response;
pub use pong_response::PongResponsePacket;

use crate::connection::McPacket;
use crate::connection::s2c::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError};
use crate::impl_clientbound_state;

#[derive(Debug)]
pub enum StatusState {
    StatusResponse(StatusResponsePacket),
    PongResponse(PongResponsePacket),
}

impl_clientbound_state! {
    state = "status";
    enum StatusState;

    match protocol {
        0.. => {
            0x00 => StatusResponse : StatusResponsePacket,
            0x01 => PongResponse : PongResponsePacket,
        }
    }
}
