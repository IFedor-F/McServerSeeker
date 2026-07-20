mod disconnect;
pub use disconnect::LoginDisconnectPacket;
mod encryption_request;
pub use encryption_request::EncryptionRequestPacket;
mod login_finished;
pub use login_finished::LoginFinishedPacket;
mod compression;
pub use compression::CompressionPacket;
mod custom_query;
pub use custom_query::CustomQueryPacket;
mod cookie_request;
pub use cookie_request::CookieRequestPacket;

use crate::connection::McPacket;
use crate::connection::s2c::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError};
use crate::impl_clientbound_state;

#[derive(Debug)]
pub enum S2CLoginState {
    LoginDisconnect(LoginDisconnectPacket),
    EncryptionRequest(EncryptionRequestPacket),
    LoginFinished(LoginFinishedPacket),
    Compression(CompressionPacket),
    CustomQuery(CustomQueryPacket),
    CookieRequest(CookieRequestPacket),
}

impl_clientbound_state! {
    state = "login";
    enum S2CLoginState;

    match protocol {
        5..=46 => {
            0x00 => LoginDisconnect: LoginDisconnectPacket,
            0x01 => EncryptionRequest: EncryptionRequestPacket,
            0x02 => LoginFinished: LoginFinishedPacket,
        },
        47..=392 => {
            0x00 => LoginDisconnect: LoginDisconnectPacket,
            0x01 => EncryptionRequest: EncryptionRequestPacket,
            0x02 => LoginFinished: LoginFinishedPacket,
            0x03 => Compression: CompressionPacket,
        },
        393..=765 => {
            0x00 => LoginDisconnect: LoginDisconnectPacket,
            0x01 => EncryptionRequest: EncryptionRequestPacket,
            0x02 => LoginFinished: LoginFinishedPacket,
            0x03 => Compression: CompressionPacket,
            0x04 => CustomQuery: CustomQueryPacket,
        },
        766..=776 => {
            0x00 => LoginDisconnect: LoginDisconnectPacket,
            0x01 => EncryptionRequest: EncryptionRequestPacket,
            0x02 => LoginFinished: LoginFinishedPacket,
            0x03 => Compression: CompressionPacket,
            0x04 => CustomQuery: CustomQueryPacket,
            0x05 => CookieRequest: CookieRequestPacket
        },
    }
}
