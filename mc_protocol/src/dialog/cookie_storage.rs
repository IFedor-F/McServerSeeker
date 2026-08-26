use crate::states::login::c2s::CookieResponsePacket;
use bytes::Bytes;
use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
#[error("Cookie storage error: data size exceeded the limit of 5 KiB, trying to save {0} bytes")]
pub struct CookieStorageError(usize);

#[derive(Debug)]
pub struct CookieStorage {
    cookies: HashMap<String, Bytes>,
    size: usize,
}

impl CookieStorage {
    const MAX_SIZE: usize = 5120; // 5 KiB (in bytes)
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
            size: 0,
        }
    }
    pub fn try_put(&mut self, key: String, data: Bytes) -> Result<(), CookieStorageError> {
        if self.size + data.len() > Self::MAX_SIZE {
            Err(CookieStorageError(self.size))
        } else {
            self.size += data.len();
            self.cookies.insert(key, data);
            Ok(())
        }
    }
    pub fn get(&self, key: &str) -> Option<&Bytes> {
        self.cookies.get(key)
    }

    pub fn format_packet(&self, key: String) -> CookieResponsePacket {
        let cookie = self.get(&key);
        match cookie {
            None => CookieResponsePacket::empty_payload(key),
            Some(payload) => CookieResponsePacket {
                key,
                payload: Some(payload.clone()),
            },
        }
    }
}
