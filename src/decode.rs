// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::{fmt, str};
use std::error::Error;

use crate::{
    internal::{ArrayBuffer, Float, Integer},
    ByteSink, ByteSource,
};

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

/// Decodes character response data
///
/// Reference: IEEE 488.2: 8.7.1 - \<CHARACTER RESPONSE DATA\>
pub fn decode_characters<S: ByteSource, T: fmt::Write>(
    decoder: &mut Decoder<S>,
    target: &mut T,
) -> Result<(), S::Error> {
    target
        .write_char(upper(decoder)? as char)
        .map_err(|_| DecodeError::BufferOverflow)?;
    loop {
        match decoder.read_byte()? {
            byte @ b'A'..=b'Z' | byte @ b'0'..=b'9' | byte @ b'_' => target
                .write_char(byte as char)
                .map_err(|_| DecodeError::BufferOverflow)?,
            byte => break decoder.end_with(byte),
        }
    }
}

#[test]
fn test_characters() {
    let test = |bytes: &'static [u8]| -> Result<String, crate::Error> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        let mut result = String::new();
        decode_characters(&mut decoder, &mut result)?;
        Ok(result)
    };

    assert_eq!(test(b"AS_DF123\n").unwrap(), "AS_DF123");
    assert!(test(b"\n").is_err());
}

/// Decodes numeric integer response data in plain (NR1), hexadecimal, octal, or binary format.
///
/// References:
///
/// - IEEE 488.2: 8.7.2 - \<NR1 NUMERIC RESPONSE DATA\>
/// - IEEE 488.2: 8.7.5 - \<HEXADECIMAL NUMERIC RESPONSE DATA\>
/// - IEEE 488.2: 8.7.6 - \<OCTAL NUMERIC RESPONSE DATA\>
/// - IEEE 488.2: 8.7.7 - \<BINARY NUMERIC RESPONSE DATA\>
pub fn decode_numeric_integer<S: ByteSource, T: Integer>(
    decoder: &mut Decoder<S>,
) -> Result<T, S::Error> {
    let mut buf = String::new();
    match decoder.read_byte()? {
        byte @ b'+' | byte @ b'-' => {
            buf.push(byte as char);
            buf.push(digit(decoder)? as char);
        }
        b'#' => match decoder.read_byte()? {
            b'H' => {
                buf.push(hex_digit(decoder)? as char);
                return loop {
                    match decoder.read_byte()? {
                        byte @ b'A'..=b'F' => buf.push(byte as char),
                        byte @ b'0'..=b'9' => buf.push(byte as char),
                        byte => {
                            decoder.end_with(byte)?;
                            break T::from_str_radix(&buf, 16)
                                .map_err(|_| DecodeError::Parse.into());
                        }
                    }
                };
            }
            b'Q' => {
                buf.push(octal_digit(decoder)? as char);
                return loop {
                    match decoder.read_byte()? {
                        byte @ b'0'..=b'7' => buf.push(byte as char),
                        byte => {
                            decoder.end_with(byte)?;
                            break T::from_str_radix(&buf, 8)
                                .map_err(|_| DecodeError::Parse.into());
                        }
                    }
                };
            }
            b'B' => {
                buf.push(binary_digit(decoder)? as char);
                return loop {
                    match decoder.read_byte()? {
                        byte @ b'0' | byte @ b'1' => buf.push(byte as char),
                        byte => {
                            decoder.end_with(byte)?;
                            break T::from_str_radix(&buf, 2)
                                .map_err(|_| DecodeError::Parse.into());
                        }
                    }
                };
            }
            _ => return Err(DecodeError::Parse)?,
        },
        byte @ b'0'..=b'9' => buf.push(byte as char),
        _ => return Err(DecodeError::Parse)?,
    }
    loop {
        match decoder.read_byte()? {
            byte @ b'0'..=b'9' => buf.push(byte as char),
            byte => {
                decoder.end_with(byte)?;
                break T::from_str_radix(&buf, 10).map_err(|_| DecodeError::Parse.into());
            }
        }
    }
}

#[test]
fn test_numeric_u8() {
    let test = |bytes: &'static [u8]| -> Result<u8, _> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        decode_numeric_integer(&mut decoder)
    };

    assert_eq!(test(b"42\n").unwrap(), 42);
    assert_eq!(test(b"#H2A\n").unwrap(), 42);
    assert_eq!(test(b"#Q52\n").unwrap(), 42);
    assert_eq!(test(b"#B101010\n").unwrap(), 42);
    assert!(test(b"-42\n").is_err());
    assert!(test(b"256\n").is_err());
}

#[test]
fn test_numeric_i8() {
    let test = |bytes: &'static [u8]| -> Result<i8, _> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        decode_numeric_integer(&mut decoder)
    };

    assert_eq!(test(b"42\n").unwrap(), 42);
    assert_eq!(test(b"-42\n").unwrap(), -42);
    assert!(test(b"-255\n").is_err());
}

/// Decodes numeric float response data in plain (NR2) or exponential (NR3) format.
///
/// References:
///
/// - IEEE 488.2: 8.7.3 - \<NR2 NUMERIC RESPONSE DATA\>
/// - IEEE 488.2: 8.7.4 - \<NR3 NUMERIC RESPONSE DATA\>
pub fn decode_numeric_float<S: ByteSource, T: Float>(
    decoder: &mut Decoder<S>,
) -> Result<T, S::Error> {
    let mut buf = String::new();
    match decoder.read_byte()? {
        byte @ b'+' | byte @ b'-' => {
            buf.push(byte as char);
            buf.push(digit(decoder)? as char);
        }
        byte @ b'0'..=b'9' => buf.push(byte as char),
        _ => return Err(DecodeError::Parse.into()),
    };
    loop {
        match decoder.read_byte()? {
            byte @ b'0'..=b'9' => buf.push(byte as char),
            byte @ b'.' => break buf.push(byte as char),
            _ => return Err(DecodeError::Parse.into()),
        }
    }
    loop {
        match decoder.read_byte()? {
            byte @ b'0'..=b'9' => buf.push(byte as char),
            byte @ b'E' => break buf.push(byte as char),
            byte => {
                decoder.end_with(byte)?;
                return T::from_str(&buf).map_err(|_| DecodeError::Parse.into());
            }
        }
    }
    buf.push(sign(decoder)? as char);
    buf.push(digit(decoder)? as char);
    loop {
        match decoder.read_byte()? {
            byte @ b'0'..=b'9' => buf.push(byte as char),
            byte => {
                decoder.end_with(byte)?;
                break T::from_str(&buf).map_err(|_| DecodeError::Parse.into());
            }
        }
    }
}

#[test]
fn test_numeric_f32() {
    let test = |bytes: &'static [u8]| -> Result<f32, _> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        decode_numeric_float(&mut decoder)
    };

    assert_eq!(test(b"42.69\n").unwrap(), 42.69);
    assert_eq!(test(b"-5.123456789\n").unwrap(), -5.123456789);
    assert_eq!(test(b"1.0005E+3\n").unwrap(), 1000.5);
    assert_eq!(test(b"-99.123E-1\n").unwrap(), -9.9123);
    assert!(test(b".1234\n").is_err());
}

/// Decodes string response data into the given target buffer.
///
/// As per IEEE 488.2, only ASCII is supported.
///
/// Reference: IEEE 488.2: 8.7.8 - \<STRING RESPONSE DATA\>
pub fn decode_string<S: ByteSource, T: fmt::Write>(
    decoder: &mut Decoder<S>,
    target: &mut T,
) -> Result<(), S::Error> {
    quote(decoder)?;
    loop {
        match decoder.read_byte()? {
            b'"' => match decoder.read_byte()? {
                b'"' => target
                    .write_char('"')
                    .map_err(|_| DecodeError::BufferOverflow)?,
                byte => break decoder.end_with(byte),
            },
            byte if byte.is_ascii() => target
                .write_char(byte as char)
                .map_err(|_| DecodeError::BufferOverflow)?,
            _ => break Err(DecodeError::Parse.into()),
        }
    }
}

#[test]
fn test_string() {
    let test = |bytes: &'static [u8]| -> Result<String, crate::Error> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        let mut result = String::new();
        decode_string(&mut decoder, &mut result)?;
        Ok(result)
    };

    assert_eq!(test(b"\"Something\n\"\n").unwrap(), "Something\n");
    assert_eq!(test(b"\"\"\"\"\n").unwrap(), "\"");
    assert!(test(b"\"broken\n").is_err());
}

/// Decodes arbitrary block response data into the given target buffer.
///
/// References:
///
/// - IEEE 488.2: 8.7.9 - \<DEFINITE LENGTH ARBITRARY BLOCK RESPONSE DATA\>
/// - IEEE 488.2: 8.7.10 - \<INDEFINITE LENGTH ARBITRARY BLOCK RESPONSE DATA\>
pub fn decode_arbitrary_block<S: ByteSource, T: ByteSink>(
    decoder: &mut Decoder<S>,
    target: &mut T,
) -> Result<(), S::Error> {
    match decoder.read_byte()? {
        b'#' => (),
        _ => return Err(DecodeError::Parse.into()),
    }
    match decoder.read_byte()? {
        byte @ b'1'..=b'9' => {
            // definite length format
            let digits = (byte - b'0') as usize;
            let mut buf = ArrayBuffer::<9>::new();
            for _ in 0..digits {
                buf.push(digit(decoder)?)
                    .map_err(|_| DecodeError::BufferOverflow)?;
            }
            let block_size = str::from_utf8(buf.finish())
                .ok()
                .and_then(|text| text.parse().ok())
                .ok_or(DecodeError::Parse)?;
            for _ in 0..block_size {
                target
                    .write_byte(decoder.read_byte()?)
                    .map_err(|_| DecodeError::BufferOverflow)?;
            }
            let byte = decoder.read_byte()?;
            decoder.end_with(byte)
        }
        b'0' => loop {
            // indefinite length format
            match decoder.read_byte()? {
                byte @ b'\n' => break decoder.end_with(byte),
                byte => target
                    .write_byte(byte)
                    .map_err(|_| DecodeError::BufferOverflow)?,
            }
        },
        _ => Err(DecodeError::Parse.into()),
    }
}

#[test]
fn test_arbitrary_block() {
    let test = |bytes: &'static [u8]| -> Result<Vec<u8>, crate::Error> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        let mut result = Vec::new();
        decode_arbitrary_block(&mut decoder, &mut result)?;
        Ok(result)
    };

    assert_eq!(test(b"#14ASDF\n").unwrap(), b"ASDF");
    assert_eq!(test(b"#210+++++?????\n").unwrap(), b"+++++?????");
    assert_eq!(test(b"#0indefinite\n").unwrap(), b"indefinite");
    assert!(test(b"#1\n").is_err());
}

/// Decodes arbitrary ASCII response data into the given target buffer.
///
/// Reference: IEEE 488.2: 8.7.11 - \<ARBITRARY ASCII RESPONSE DATA\>
pub fn decode_arbitrary_ascii<S: ByteSource, T: fmt::Write>(
    decoder: &mut Decoder<S>,
    target: &mut T,
) -> Result<(), S::Error> {
    loop {
        match decoder.read_byte()? {
            byte @ b'\n' => break decoder.end_with(byte),
            byte if byte.is_ascii() => target
                .write_char(byte as char)
                .map_err(|_| DecodeError::BufferOverflow)?,
            _ => break Err(DecodeError::Parse.into()),
        }
    }
}

/// Decodes boolean response data.
///
/// IEEE 488.2 does not formally specify a response format for booleans, but commands with boolean
/// responses tend to use NR1 numerical literals 0 and 1, which match the SCPI boolean format spec.
///
/// Reference: SCPI 1999.0: 7.3 - Boolean Program Data
pub fn decode_boolean<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<bool, S::Error> {
    match decoder.read_byte()? {
        b'0' => {
            let byte = decoder.read_byte()?;
            decoder.end_with(byte)?;
            Ok(false)
        }
        b'1' => {
            let byte = decoder.read_byte()?;
            decoder.end_with(byte)?;
            Ok(true)
        }
        _ => Err(DecodeError::Parse.into()),
    }
}

#[cfg(test)]
mod boolean_decoding {
    use super::{decode_boolean, Decoder};
    use crate::{decode::DecodeError, Error};

    #[test]
    fn zero_is_false() {
        match decode(b"0\n") {
            Ok(false) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn one_is_true() {
        match decode(b"1\n") {
            Ok(true) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn textual_forms_are_not_valid() {
        match decode(b"false\n") {
            Err(Error::Decode(DecodeError::Parse)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
        match decode(b"true\n") {
            Err(Error::Decode(DecodeError::Parse)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    fn decode(bytes: &'static [u8]) -> Result<bool, Error> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        decode_boolean(&mut decoder)
    }
}
