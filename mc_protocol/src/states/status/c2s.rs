mod ping_request_packet;
pub use ping_request_packet::PingRequestPacket;
mod status_request_packet;
pub use status_request_packet::StatusRequestPacket;

use crate::connection::c2s::{ServerBoundPacket, ServerBoundState};
use crate::impl_serverbound_state;

#[derive(Debug)]
pub enum StatusState {
    StatusRequest(StatusRequestPacket),
    PingRequest(PingRequestPacket),

}

impl_serverbound_state! {
    state = "status";
    enum StatusState;
    match protocol {
        5..=776 => {
            0x00 => StatusRequest: StatusRequestPacket,
            0x01 => PingRequest: PingRequestPacket,
        },
    }
}
