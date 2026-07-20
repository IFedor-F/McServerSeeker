mod hello;
pub use hello::HelloPacket;
mod custom_query_response;
pub use custom_query_response::CustomQueryAnswerPacket;
mod login_acknowledged;
pub use login_acknowledged::LoginAcknowledgedPacket;
mod cookie_response;
pub use cookie_response::CookieResponsePacket;
mod key;
pub use key::KeyPacket;

use crate::connection::c2s::{ServerBoundPacket, ServerBoundState};
use crate::impl_serverbound_state;

#[derive(Debug)]
pub enum C2SLoginState {
    Hello(HelloPacket),
    Key(KeyPacket),
    CustomQueryAnswer(CustomQueryAnswerPacket),
    LoginAcknowledged(LoginAcknowledgedPacket),
    CookieResponse(CookieResponsePacket),
}

impl_serverbound_state! {
    state = "login";
    enum C2SLoginState;
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
