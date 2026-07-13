mod disconnect_packet;
pub use disconnect_packet::DisconnectPacket;
mod encryption_request_packet;
pub use encryption_request_packet::EncryptionRequestPacket;
mod login_finished_packet;
pub use login_finished_packet::LoginFinishedPacket;
mod compression_packet;
pub use compression_packet::CompressionPacket;
mod custom_query_packet;
pub use custom_query_packet::CustomQueryPacket;
mod cookie_request_packet;
pub use cookie_request_packet::CookieRequestPacket;

use crate::connection::McPacket;
use crate::connection::s2c::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError};
use crate::impl_clientbound_state;

#[derive(Debug)]
pub enum LoginState {
    Disconnect(DisconnectPacket),
    EncryptionRequest(EncryptionRequestPacket),
    LoginFinished(LoginFinishedPacket),
    Compression(CompressionPacket),
    LoginPluginRequest(CustomQueryPacket),
    CookieRequest(CookieRequestPacket),
}

impl_clientbound_state! {
    state = "login";
    enum LoginState;

    match protocol {
        5..=46 => {
            0x00 => Disconnect: DisconnectPacket,
            0x01 => EncryptionRequest: EncryptionRequestPacket,
            0x02 => LoginFinished: LoginFinishedPacket,
        },
        47..=392 => {
            0x00 => Disconnect: DisconnectPacket,
            0x01 => EncryptionRequest: EncryptionRequestPacket,
            0x02 => LoginFinished: LoginFinishedPacket,
            0x03 => Compression: CompressionPacket,
        },
        393..=765 => {
            0x00 => Disconnect: DisconnectPacket,
            0x01 => EncryptionRequest: EncryptionRequestPacket,
            0x02 => LoginFinished: LoginFinishedPacket,
            0x03 => Compression: CompressionPacket,
            0x04 => LoginPluginRequest: CustomQueryPacket,
        },
        766..=776 => {
            0x00 => Disconnect: DisconnectPacket,
            0x01 => EncryptionRequest: EncryptionRequestPacket,
            0x02 => LoginFinished: LoginFinishedPacket,
            0x03 => Compression: CompressionPacket,
            0x04 => LoginPluginRequest: CustomQueryPacket,
            0x05 => CookieRequest: CookieRequestPacket
        },
    }
}
