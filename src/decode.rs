// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt;
use std::error::Error;

pub use self::arbitrary_ascii::*;
pub use self::arbitrary_block::*;
pub use self::boolean::*;
pub use self::characters::*;
pub use self::numeric_float::*;
pub use self::numeric_integer::*;
pub use self::string::*;
use crate::ByteSource;

mod arbitrary_ascii;
mod arbitrary_block;
mod boolean;
mod characters;
mod numeric_float;
mod numeric_integer;
mod string;

#[derive(Debug)]
pub enum DecodeError {
    Parse,
    UnexpectedEnd,
    BufferOverflow,
    InvalidDecodeState(DecodeState),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeError::Parse => write!(f, "parse error"),
            DecodeError::UnexpectedEnd => write!(f, "unexpected end"),
            DecodeError::BufferOverflow => write!(f, "buffer overflow"),
            DecodeError::InvalidDecodeState(state) => {
                write!(f, "invalid decode state ({:?})", state)
            }
        }
    }
}

impl Error for DecodeError {}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DecodeState {
    Initial,
    Data,
    DataExpected,
    MessageUnitExpected,
    End,
}

impl Default for DecodeState {
    fn default() -> Self {
        DecodeState::Initial
    }
}

#[must_use]
pub struct Decoder<S: ByteSource> {
    source: S,
    state: DecodeState,
    peeked: Option<u8>,
}

impl<S: ByteSource> Decoder<S> {
    pub fn new(source: S) -> Decoder<S> {
        Decoder {
            source,
            state: DecodeState::default(),
            peeked: None,
        }
    }
    pub fn read_byte(&mut self) -> Result<u8, S::Error> {
        if let Some(byte) = self.peeked.take() {
            Ok(byte)
        } else {
            let byte = self.source.read_byte()?;
            Ok(byte)
        }
    }
    pub fn peek_byte(&mut self) -> Result<u8, S::Error> {
        if let Some(byte) = self.peeked {
            Ok(byte)
        } else {
            let byte = self.source.read_byte()?;
            self.peeked = Some(byte);
            Ok(byte)
        }
    }
    fn skip_whitespace(&mut self) -> Result<(), S::Error> {
        self.peeked = Some(loop {
            match self.read_byte()? {
                // Reference: IEEE 488.2 7.4.1.2 - Encoding Syntax
                0x00..=0x09 | 0x0b..=0x20 => (),
                byte => break byte,
            }
        });
        Ok(())
    }
    pub fn begin_response_data(&mut self) -> Result<(), S::Error> {
        match self.state {
            DecodeState::Initial | DecodeState::DataExpected | DecodeState::MessageUnitExpected => {
                self.skip_whitespace()?;
                self.state = DecodeState::Data;
                Ok(())
            }
            _ => Err(DecodeError::InvalidDecodeState(self.state).into()),
        }
    }
    pub fn end_with(&mut self, byte: u8) -> Result<(), S::Error> {
        self.state = match self.state {
            DecodeState::Data => match byte {
                // Reference: IEEE 488.2: 8.5 - \<RESPONSE MESSAGE TERMINATOR\>
                b'\n' => DecodeState::End,
                // Reference: IEEE 488.2: 8.4.1 - \<RESPONSE MESSAGE UNIT SEPARATOR\>
                b';' => DecodeState::MessageUnitExpected,
                // Reference: IEEE 488.2: 8.4.2 - \<RESPONSE DATA SEPARATOR\>
                b',' => DecodeState::DataExpected,
                _ => return Err(DecodeError::InvalidDecodeState(self.state))?,
            },
            _ => return Err(DecodeError::InvalidDecodeState(self.state))?,
        };
        Ok(())
    }
    pub fn is_at_end(&self) -> bool {
        self.state == DecodeState::End
    }
    pub fn finish(self) -> Result<S, S::Error> {
        match self.state {
            DecodeState::End => Ok(self.source),
            _ => Err(DecodeError::InvalidDecodeState(self.state).into()),
        }
    }
}

#[inline]
fn sign<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<u8, S::Error> {
    match decoder.read_byte()? {
        byte @ b'-' | byte @ b'+' => Ok(byte),
        _ => Err(DecodeError::Parse.into()),
    }
}

#[inline]
fn digit<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<u8, S::Error> {
    match decoder.read_byte()? {
        byte @ b'0'..=b'9' => Ok(byte),
        _ => Err(DecodeError::Parse.into()),
    }
}

#[inline]
fn hex_digit<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<u8, S::Error> {
    match decoder.read_byte()? {
        byte @ b'A'..=b'F' => Ok(byte),
        byte @ b'0'..=b'9' => Ok(byte),
        _ => Err(DecodeError::Parse.into()),
    }
}

#[inline]
fn octal_digit<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<u8, S::Error> {
    match decoder.read_byte()? {
        byte @ b'0'..=b'7' => Ok(byte),
        _ => Err(DecodeError::Parse.into()),
    }
}

#[inline]
fn binary_digit<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<u8, S::Error> {
    match decoder.read_byte()? {
        byte @ b'0'..=b'1' => Ok(byte),
        _ => Err(DecodeError::Parse.into()),
    }
}

#[inline]
fn upper<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<u8, S::Error> {
    match decoder.read_byte()? {
        byte @ b'A'..=b'Z' => Ok(byte),
        _ => Err(DecodeError::Parse.into()),
    }
}

#[inline]
fn quote<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<u8, S::Error> {
    match decoder.read_byte()? {
        byte @ b'"' => Ok(byte),
        _ => Err(DecodeError::Parse.into()),
    }
}
