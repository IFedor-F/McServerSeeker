mod ping_request;
pub use ping_request::PingRequestPacket;
mod status_request;
pub use status_request::StatusRequestPacket;

use crate::connection::c2s::{ServerBoundPacket, ServerBoundState};
use crate::impl_serverbound_state;

#[derive(Debug)]
pub enum C2SStatusState {
    StatusRequest(StatusRequestPacket),
    PingRequest(PingRequestPacket),

}

impl_serverbound_state! {
    state = "status";
    enum C2SStatusState;
    match protocol {
        5..=776 => {
            0x00 => StatusRequest: StatusRequestPacket,
            0x01 => PingRequest: PingRequestPacket,
        },
    }
}
