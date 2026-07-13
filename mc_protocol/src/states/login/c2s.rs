mod hello_packet;
pub use hello_packet::HelloPacket;
mod custom_query_response_packet;
pub use custom_query_response_packet::CustomQueryAnswerPacket;
mod login_acknowledged_packet;
pub use login_acknowledged_packet::LoginAcknowledgedPacket;
mod cookie_response_packet;
pub use cookie_response_packet::CookieResponsePacket;
mod key_packet;
pub use key_packet::KeyPacket;

use crate::connection::c2s::{ServerBoundPacket, ServerBoundState};
use crate::impl_serverbound_state;

#[derive(Debug)]
pub enum LoginState {
    Hello(HelloPacket),
    Key(KeyPacket),
    CustomQueryAnswer(CustomQueryAnswerPacket),
    LoginAcknowledged(LoginAcknowledgedPacket),
    CookieResponse(CookieResponsePacket),
}

impl_serverbound_state! {
    state = "login";
    enum LoginState;
    match protocol {
        5..=392 => {
            0x00 => Hello: HelloPacket,
            0x01 => Key: KeyPacket,
        },
        393..=763 => {
            0x00 => Hello: HelloPacket,
            0x01 => Key: KeyPacket,
            0x02 => CustomQueryAnswer: CustomQueryAnswerPacket,
        },
        764..=765 => {
            0x00 => Hello: HelloPacket,
            0x01 => Key: KeyPacket,
            0x02 => CustomQueryAnswer: CustomQueryAnswerPacket,
            0x03 => LoginAcknowledged: LoginAcknowledgedPacket,
        },
        766..=776 => {
            0x00 => Hello: HelloPacket,
            0x01 => Key: KeyPacket,
            0x02 => CustomQueryAnswer: CustomQueryAnswerPacket,
            0x03 => LoginAcknowledged: LoginAcknowledgedPacket,
            0x04 => CookieResponse: CookieResponsePacket,
        },
    }
}
