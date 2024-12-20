use crate::{DiscordError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::Serialize;
use std::io::{Read, Write};

pub(crate) const MAX_RPC_FRAME_SIZE: usize = 64 * 1024;

/// Codes for payload types
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum OpCode {
    /// Handshake payload
    Handshake = 0,
    /// Frame payload
    Frame = 1,
    /// Close payload
    Close = 2,
    /// Ping payload
    Ping = 3,
    /// Pong payload
    Pong = 4,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
/// Header for the payload
///
/// Determines the length of the payload, and the type of payload
pub struct FrameHeader {
    /// The opcode for the payload
    opcode: OpCode,
    /// The length of the payload
    length: u32,
}

impl FrameHeader {
    #[must_use]
    /// Convert an array of bytes to a [`FrameHeader`]
    ///
    /// # Safety
    /// This reinterprets the bytes as a [`FrameHeader`]. It is up to the caller to ensure that the
    /// bytes are valid.
    pub unsafe fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != std::mem::size_of::<FrameHeader>() {
            return None;
        }

        Some(unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast()) })
    }

    #[must_use]
    /// Get the expected message length
    pub fn message_length(&self) -> u32 {
        self.length
    }

    #[must_use]
    /// Get the opcode
    pub fn opcode(&self) -> OpCode {
        self.opcode
    }
}

// NOTE: Currently unused
// Probably remove in future
// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// #[repr(C)]
// /// Frame passed over the socket
// ///
// /// Contains the header and the payload
// pub struct Frame {
//     /// The header for the payload
//     header: FrameHeader,
//     /// The actual payload
//     message: [std::os::raw::c_char; MAX_RPC_FRAME_SIZE - std::mem::size_of::<FrameHeader>()],
// }

// impl From<Frame> for Message {
//     fn from(header: Frame) -> Self {
//         Self {
//             opcode: header.header.opcode,
//             payload: unsafe { CStr::from_ptr(header.message.as_ptr()) }
//                 .to_string_lossy()
//                 .into_owned(),
//         }
//     }
// }

/// Message struct for the Discord RPC
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Message {
    /// The payload type for this `Message`
    pub opcode: OpCode,
    /// The actual payload
    pub payload: String,
}

impl Message {
    /// Create a new `Message`
    ///
    /// # Errors
    /// - Could not serialize the payload
    pub fn new<T>(opcode: OpCode, payload: T) -> Result<Self>
    where
        T: Serialize,
    {
        Ok(Self {
            opcode,
            payload: serde_json::to_string(&payload)?,
        })
    }

    /// Encode message
    ///
    /// # Errors
    /// - Failed to write to the buffer
    ///
    /// # Panics
    /// - The payload length is not a 32 bit number
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![];

        let payload_length = u32::try_from(self.payload.len()).expect("32-bit payload length");

        bytes.write_u32::<LittleEndian>(self.opcode as u32)?;
        bytes.write_u32::<LittleEndian>(payload_length)?;
        bytes.write_all(self.payload.as_bytes())?;

        Ok(bytes)
    }

    /// Decode message
    ///
    /// # Errors
    /// - Failed to read from buffer
    pub fn decode(mut bytes: &[u8]) -> Result<Self> {
        let opcode =
            OpCode::from_u32(bytes.read_u32::<LittleEndian>()?).ok_or(DiscordError::Conversion)?;
        let len = bytes.read_u32::<LittleEndian>()? as usize;
        let mut payload = String::with_capacity(len);
        bytes.read_to_string(&mut payload)?;

        Ok(Self { opcode, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Something {
        empty: bool,
    }

    #[test]
    fn test_encoder() {
        let msg = Message::new(OpCode::Frame, Something { empty: true })
            .expect("Failed to serialize message");
        let encoded = msg.encode().expect("Failed to encode message");
        let decoded = Message::decode(&encoded).expect("Failed to decode message");
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_opcode() {
        assert_eq!(OpCode::from_u32(0), Some(OpCode::Handshake));
        assert_eq!(OpCode::from_u32(4), Some(OpCode::Pong));
        assert_eq!(OpCode::from_u32(5), None);
    }
}
