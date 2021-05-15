// SPDX-FileCopyrightText: 2020-2021 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt::{self, Write};
use std::error::Error;

use crate::{
    internal::{ArrayBuffer, Float, Integer},
    ByteSink,
};

#[derive(Debug)]
pub enum EncodeError<E>
where
    E: fmt::Debug + fmt::Display,
{
    NonAsciiString,
    BlockSizeOverflow(usize),
    InvalidEncodeState(EncodeState),
    Other(E),
}

impl<T> fmt::Display for EncodeError<T>
where
    T: fmt::Debug + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncodeError::NonAsciiString => write!(f, "invalid non-ascii string"),
            EncodeError::BlockSizeOverflow(size) => {
                write!(f, "block size {} overflows protocol limit", size)
            }
            EncodeError::InvalidEncodeState(state) => {
                write!(f, "invalid encode state ({:?})", state)
            }
            EncodeError::Other(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl<T> Error for EncodeError<T> where T: fmt::Debug + fmt::Display {}

impl<T> From<T> for EncodeError<T>
where
    T: fmt::Debug + fmt::Display,
{
    fn from(err: T) -> EncodeError<T> {
        EncodeError::Other(err)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EncodeState {
    Initial,
    Header,
    Data,
    End,
}

impl Default for EncodeState {
    fn default() -> Self {
        EncodeState::Initial
    }
}

#[must_use]
pub struct Encoder<S: ByteSink> {
    sink: S,
    state: EncodeState,
}

/// Reference: IEEE 488.2: 7.4.1 - \<PROGRAM MESSAGE UNIT SEPARATOR\>
const PROGRAM_MESSAGE_UNIT_SEPARATOR: u8 = b';';

/// Reference: IEEE 488.2: 7.4.2 - \<PROGRAM DATA SEPARATOR\>
const PROGRAM_DATA_SEPARATOR: u8 = b',';

/// Reference: IEEE 488.2: 7.4.3 - \<PROGRAM HEADER SEPARATOR\>
const PROGRAM_HEADER_SEPARATOR: u8 = b' ';

/// Reference: IEEE 488.2: 7.5 - \<PROGRAM MESSAGE TERMINATOR\>
const PROGRAM_MESSAGE_TERMINATOR: u8 = b'\n';

impl<S: ByteSink> Encoder<S> {
    pub fn new(sink: S) -> Encoder<S> {
        Encoder {
            sink,
            state: EncodeState::default(),
        }
    }
    pub fn write_byte(&mut self, byte: u8) -> Result<(), EncodeError<S::Error>> {
        debug_assert!(self.state == EncodeState::Header || self.state == EncodeState::Data);
        self.sink.write_byte(byte)?;
        Ok(())
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError<S::Error>> {
        debug_assert!(self.state == EncodeState::Header || self.state == EncodeState::Data);
        self.sink.write_bytes(bytes)?;
        Ok(())
    }
    pub fn begin_message_unit(&mut self) -> Result<(), EncodeError<S::Error>> {
        self.state = match self.state {
            EncodeState::Initial => EncodeState::Header,
            EncodeState::Header | EncodeState::Data => {
                self.sink.write_byte(PROGRAM_MESSAGE_UNIT_SEPARATOR)?;
                EncodeState::Header
            }
            _ => return Err(EncodeError::InvalidEncodeState(self.state)),
        };
        Ok(())
    }
    pub fn begin_program_data(&mut self) -> Result<(), EncodeError<S::Error>> {
        self.state = match self.state {
            EncodeState::Header => {
                self.sink.write_byte(PROGRAM_HEADER_SEPARATOR)?;
                EncodeState::Data
            }
            EncodeState::Data => {
                self.sink.write_byte(PROGRAM_DATA_SEPARATOR)?;
                EncodeState::Data
            }
            _ => return Err(EncodeError::InvalidEncodeState(self.state)),
        };
        Ok(())
    }
    pub fn end_message(&mut self) -> Result<(), EncodeError<S::Error>> {
        self.state = match self.state {
            EncodeState::Header | EncodeState::Data => {
                self.sink.write_byte(PROGRAM_MESSAGE_TERMINATOR)?;
                EncodeState::End
            }
            EncodeState::End => EncodeState::End,
            _ => return Err(EncodeError::InvalidEncodeState(self.state)),
        };
        Ok(())
    }
    pub fn finish(mut self) -> Result<S, EncodeError<S::Error>> {
        self.end_message()?;
        Ok(self.sink)
    }
}

/// Encodes an integer value into bytes according to IEEE 488.2.
///
/// Reference: IEEE 488.2: 7.7.2 - \<DECIMAL NUMERIC PROGRAM DATA\>
pub fn encode_numeric_integer<S: ByteSink, T: Integer>(
    encoder: &mut Encoder<S>,
    value: T,
) -> Result<(), EncodeError<S::Error>> {
    let mut fmt: ArrayBuffer<32> = ArrayBuffer::new();
    let res = write!(&mut fmt, "{}", value);
    debug_assert_eq!(res, Ok(()));
    encoder.write_bytes(fmt.finish())?;
    Ok(())
}

/// Encodes a floating point value into bytes according to SCPI 1999.0 / IEEE 488.2.
///
/// References:
///   - IEEE 488.2: 7.7.2 - \<DECIMAL NUMERIC PROGRAM DATA\>
///   - SCPI 1999.0: 7.2 - Decimal Numeric Program Data
pub fn encode_numeric_float<S: ByteSink, T: Float>(
    encoder: &mut Encoder<S>,
    value: T,
) -> Result<(), EncodeError<S::Error>> {
    // TODO: consider validating the range?
    if value.is_finite() {
        let mut fmt: ArrayBuffer<64> = ArrayBuffer::new();
        let res = write!(&mut fmt, "{:E}", value);
        debug_assert_eq!(res, Ok(()));
        encoder.write_bytes(fmt.finish())?;
    } else if value.is_nan() {
        // SCPI 1999.0: 7.2.1.5 - Not A Number (NAN)
        encoder.write_bytes(b"NAN")?;
    } else {
        // SCPI 1999.0: 7.2.1.4 - INFinity and Negative INFinity (NINF)
        if value.is_sign_positive() {
            encoder.write_bytes(b"INF")?;
        } else {
            encoder.write_bytes(b"NINF")?;
        }
    }
    Ok(())
}

/// Encodes an ASCII string into bytes according to IEEE 488.2.
///
/// Reference: IEEE 488.2: 7.7.5 - \<STRING PROGRAM DATA\>
pub fn encode_string<S: ByteSink>(
    encoder: &mut Encoder<S>,
    data: &str,
) -> Result<(), EncodeError<S::Error>> {
    if data.as_bytes().iter().all(|ch| ch.is_ascii()) {
        // IEEE 488.2: 7.7.5.2 - Encoding syntax
        encoder.write_byte(b'"')?;
        let mut chunk_iter = data.as_bytes().split(|&ch| ch == b'"').peekable();
        while let Some(chunk) = chunk_iter.next() {
            encoder.write_bytes(chunk)?;
            if chunk_iter.peek().is_some() {
                encoder.write_bytes(b"\"\"")?;
            }
        }
        encoder.write_byte(b'"')?;
        Ok(())
    } else {
        Err(EncodeError::NonAsciiString)
    }
}

/// Encodes a slice of bytes into definite arbitrary block bytes according to IEEE 488.2.
///
/// Reference: IEEE 488.2: 7.7.6 - \<ARBITRARY BLOCK PROGRAM DATA\>
pub fn encode_definite_block<S: ByteSink>(
    encoder: &mut Encoder<S>,
    data: &[u8],
) -> Result<(), EncodeError<S::Error>> {
    let mut fmt: ArrayBuffer<11> = ArrayBuffer::new();

    // IEEE 488.2: 7.7.6.2 - Encoding syntax
    write!(&mut fmt, "#0{}", data.len()).map_err(|_| EncodeError::BlockSizeOverflow(data.len()))?;
    let header = fmt.finish();
    let digits = header[2..].len();
    header[1] = b'0' + (digits as u8);
    encoder.write_bytes(header)?;
    encoder.write_bytes(data)?;
    Ok(())
}
