use super::{
    MCJsonTextFieldError, McJsonTextComponent, McJsonTextField, McNbtFieldError, McReadBuf,
    McTextComponent,
};
use bytes::{Bytes};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McChatError {
    #[error("McChat error: failed to parse McJsonText: {0}")]
    McJsonTextComponentError(#[from] MCJsonTextFieldError),

    #[error("McChat error: failed to parse McTextComponent: {0}")]
    McTextComponentError(#[from] McNbtFieldError),
}

#[derive(Debug)]
pub enum McChat {
    JsonTextComponent(McJsonTextComponent),
    TextComponent(McTextComponent),
}

impl McChat {
    pub fn read_from_buf_with_protocol(
        buf: &mut Bytes,
        protocol: i32,
    ) -> Result<Self, McChatError> {
        match protocol {
            ..=764 => {
                let text = McJsonTextField::<262144>::read_from_buf(buf)?;
                Ok(McChat::JsonTextComponent(text))
            }
            765.. => {
                let text = McTextComponent::read_from_buf_with_protocol(buf, protocol)?;
                Ok(McChat::TextComponent(text))
            }
        }
    }
    pub fn formatted(&self) -> String {
        match self {
            McChat::JsonTextComponent(text) => {
                text.formatted()
            }
            McChat::TextComponent(text) => {
                text.formatted()
            }
        }
    }
}
