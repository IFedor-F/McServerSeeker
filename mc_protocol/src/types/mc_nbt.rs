use crate::types::McReadBuf;
use bytes::{Buf, Bytes};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McNbtFieldError {
    #[error("Not enough bytes in the buffer to read NBT")]
    UnexpectedEof,

    #[error("Unknown NBT Tag ID: {0}")]
    UnknownTagId(u8),

    #[error("Invalid UTF-8 string in NBT: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("I/O error: {0}")]
    Io(std::io::Error),

    #[error("Invalid NBT List length: {0}")]
    InvalidListLength(i32),

    #[error("Nbt structure is too deep")]
    NbtTooDeep,
}

impl From<std::io::Error> for McNbtFieldError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::UnexpectedEof {
            Self::UnexpectedEof
        } else {
            Self::Io(err)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum McNbtTag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<McNbtTag>),
    Compound(HashMap<String, McNbtTag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}
const MAX_PREALLOC_CAPACITY: usize = 1024;
const MAX_NBT_DEPTH: usize = 512;

#[derive(Debug)]
pub struct McLegacyNbtField;
impl McReadBuf for McLegacyNbtField {
    type Output = McNbtTag;
    type Error = McNbtFieldError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        if !buf.has_remaining() {
            return Err(McNbtFieldError::UnexpectedEof);
        }

        let root_tag_id = buf.get_u8();
        if root_tag_id == 0 {
            return Ok(McNbtTag::End);
        }

        if buf.remaining() < 2 {
            return Err(McNbtFieldError::UnexpectedEof);
        }
        let name_len = buf.get_u16();
        if buf.remaining() < name_len as usize {
            return Err(McNbtFieldError::UnexpectedEof);
        }
        buf.advance(name_len as usize);

        read_nbt_payload(root_tag_id, buf, 0)
    }
}
#[derive(Debug)]
pub struct McModernNbtField;

impl McReadBuf for McModernNbtField {
    type Output = McNbtTag;
    type Error = McNbtFieldError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        if !buf.has_remaining() {
            return Err(McNbtFieldError::UnexpectedEof);
        }

        let root_tag_id = buf.get_u8();
        if root_tag_id == 0 {
            return Ok(McNbtTag::End);
        }

        read_nbt_payload(root_tag_id, buf, 0)
    }
}

fn read_nbt_payload(
    tag_id: u8,
    buf: &mut Bytes,
    depth: usize,
) -> Result<McNbtTag, McNbtFieldError> {
    if depth > MAX_NBT_DEPTH {
        return Err(McNbtFieldError::NbtTooDeep);
    }

    match tag_id {
        0 => Ok(McNbtTag::End),
        1 => {
            if !buf.has_remaining() {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            Ok(McNbtTag::Byte(buf.get_i8()))
        }
        2 => {
            if buf.remaining() < 2 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            Ok(McNbtTag::Short(buf.get_i16()))
        }
        3 => {
            if buf.remaining() < 4 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            Ok(McNbtTag::Int(buf.get_i32()))
        }
        4 => {
            if buf.remaining() < 8 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            Ok(McNbtTag::Long(buf.get_i64()))
        }
        5 => {
            if buf.remaining() < 4 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            Ok(McNbtTag::Float(buf.get_f32()))
        }
        6 => {
            if buf.remaining() < 8 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            Ok(McNbtTag::Double(buf.get_f64()))
        }
        7 => {
            // TAG_Byte_Array
            if buf.remaining() < 4 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            let len = buf.get_i32();
            if len < 0 || buf.remaining() < len as usize {
                return Err(McNbtFieldError::UnexpectedEof);
            }

            // OOM isn't possible because we checks buf.remaining()
            let mut arr = Vec::with_capacity(len as usize);
            for _ in 0..len {
                arr.push(buf.get_i8());
            }
            Ok(McNbtTag::ByteArray(arr))
        }
        8 => {
            // TAG_String
            let s = read_nbt_string(buf)?;
            Ok(McNbtTag::String(s))
        }
        9 => {
            // TAG_List
            if buf.remaining() < 5 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            let item_id = buf.get_u8();
            let len = buf.get_i32();

            if len <= 0 {
                return Ok(McNbtTag::List(Vec::new()));
            }

            // OOM protection: doesn't pre-allocate more than MAX_PREALLOC_CAPACITY
            let safe_capacity = (len as usize).min(MAX_PREALLOC_CAPACITY);
            let mut list = Vec::with_capacity(safe_capacity);

            for _ in 0..len {
                list.push(read_nbt_payload(item_id, buf, depth + 1)?);
            }
            Ok(McNbtTag::List(list))
        }
        10 => {
            // TAG_Compound
            let mut map = HashMap::new();
            loop {
                if !buf.has_remaining() {
                    return Err(McNbtFieldError::UnexpectedEof);
                }
                let item_id = buf.get_u8();

                if item_id == 0 {
                    break;
                }

                let name = read_nbt_string(buf)?;
                let value = read_nbt_payload(item_id, buf, depth + 1)?;
                map.insert(name, value);
            }
            Ok(McNbtTag::Compound(map))
        }
        11 => {
            // TAG_Int_Array
            if buf.remaining() < 4 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            let len = buf.get_i32();
            if len < 0 || buf.remaining() < (len as usize * 4) {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            let mut arr = Vec::with_capacity(len as usize);
            for _ in 0..len {
                arr.push(buf.get_i32());
            }
            Ok(McNbtTag::IntArray(arr))
        }
        12 => {
            // TAG_Long_Array
            if buf.remaining() < 4 {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            let len = buf.get_i32();
            if len < 0 || buf.remaining() < (len as usize * 8) {
                return Err(McNbtFieldError::UnexpectedEof);
            }
            let mut arr = Vec::with_capacity(len as usize);
            for _ in 0..len {
                arr.push(buf.get_i64());
            }
            Ok(McNbtTag::LongArray(arr))
        }
        _ => Err(McNbtFieldError::UnknownTagId(tag_id)),
    }
}

fn read_nbt_string(buf: &mut Bytes) -> Result<String, McNbtFieldError> {
    if buf.remaining() < 2 {
        return Err(McNbtFieldError::UnexpectedEof);
    }
    let len = buf.get_u16() as usize;
    if buf.remaining() < len {
        return Err(McNbtFieldError::UnexpectedEof);
    }

    let string_bytes = buf.split_to(len);
    let s = String::from_utf8(string_bytes.to_vec())?;
    Ok(s)
}

#[derive(Debug)]
pub struct McTextComponent(pub McNbtTag);

impl McTextComponent {
    pub fn read_from_buf_with_protocol(buf: &mut Bytes, protocol: i32) -> Result<Self, McNbtFieldError> {
        Ok(Self(read_nbt_from_buf(buf, protocol)?))
    }
    fn extract_raw_text(value: &McNbtTag, out: &mut String) {
        let mut stack = vec![value];

        while let Some(current) = stack.pop() {
            match current {
                McNbtTag::String(s) => {
                    out.push_str(s);
                }
                McNbtTag::List(arr) => {
                    for item in arr.iter().rev() {
                        stack.push(item);
                    }
                }
                McNbtTag::Compound(obj) => {
                    if let Some(extra) = obj.get("extra") {
                        stack.push(extra);
                    }
                    if let Some(McNbtTag::String(text)) = obj.get("text") {
                        out.push_str(text);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn formatted(&self) -> String {
        let mut result = String::new();
        Self::extract_raw_text(&self.0, &mut result);
        result
    }
}

#[inline(always)]
pub fn read_nbt_from_buf(
    buf: &mut Bytes,
    protocol: i32,
) -> Result<McNbtTag, McNbtFieldError> {
    match protocol {
        ..764 => McLegacyNbtField::read_from_buf(buf),
        764.. => McModernNbtField::read_from_buf(buf),
    }
}
