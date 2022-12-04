// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt::{self, Write};

use crate::{
    internal::{ArrayBuffer, Float, Integer},
    is_program_mnemonic, ByteSink,
};

#[derive(Debug)]
pub enum EncodeError {
    NonAsciiString,
    InvalidCharacterData,
    BlockSizeOverflow(usize),
    InvalidEncodeState(EncodeState),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncodeError::InvalidCharacterData => write!(f, "invalid character data"),
            EncodeError::NonAsciiString => write!(f, "invalid non-ascii string"),
            EncodeError::BlockSizeOverflow(size) => {
                write!(f, "block size {} overflows protocol limit", size)
            }
            EncodeError::InvalidEncodeState(state) => {
                write!(f, "invalid encode state ({:?})", state)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {}

/// A sink for encoded bytes
pub trait EncodeSink: ByteSink {
    fn terminate_message(&mut self) -> Result<(), Self::Error> {
        self.write_byte(PROGRAM_MESSAGE_TERMINATOR)
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
#[derive(Copy, Clone, Debug)]
pub struct Encoder<S: EncodeSink> {
    sink: S,
    state: EncodeState,
}

/// Reference: IEEE 488.2: 7.4.1 - \<PROGRAM MESSAGE UNIT SEPARATOR\>
pub const PROGRAM_MESSAGE_UNIT_SEPARATOR: u8 = b';';

/// Reference: IEEE 488.2: 7.4.2 - \<PROGRAM DATA SEPARATOR\>
pub const PROGRAM_DATA_SEPARATOR: u8 = b',';

/// Reference: IEEE 488.2: 7.4.3 - \<PROGRAM HEADER SEPARATOR\>
pub const PROGRAM_HEADER_SEPARATOR: u8 = b' ';

/// Reference: IEEE 488.2: 7.5 - \<PROGRAM MESSAGE TERMINATOR\>
pub const PROGRAM_MESSAGE_TERMINATOR: u8 = b'\n';

impl<S: EncodeSink> Encoder<S> {
    pub fn new(sink: S) -> Encoder<S> {
        Encoder {
            sink,
            state: EncodeState::default(),
        }
    }
    pub fn write_byte(&mut self, byte: u8) -> Result<(), S::Error> {
        debug_assert!(self.state == EncodeState::Header || self.state == EncodeState::Data);
        self.sink.write_byte(byte)?;
        Ok(())
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), S::Error> {
        debug_assert!(self.state == EncodeState::Header || self.state == EncodeState::Data);
        self.sink.write_bytes(bytes)?;
        Ok(())
    }
    pub fn begin_message_unit(&mut self) -> Result<(), S::Error> {
        self.state = match self.state {
            EncodeState::Initial => EncodeState::Header,
            EncodeState::Header | EncodeState::Data => {
                self.sink.write_byte(PROGRAM_MESSAGE_UNIT_SEPARATOR)?;
                EncodeState::Header
            }
            _ => return Err(EncodeError::InvalidEncodeState(self.state).into()),
        };
        Ok(())
    }
    pub fn begin_program_data(&mut self) -> Result<(), S::Error> {
        self.state = match self.state {
            EncodeState::Header => {
                self.sink.write_byte(PROGRAM_HEADER_SEPARATOR)?;
                EncodeState::Data
            }
            EncodeState::Data => {
                self.sink.write_byte(PROGRAM_DATA_SEPARATOR)?;
                EncodeState::Data
            }
            _ => return Err(EncodeError::InvalidEncodeState(self.state).into()),
        };
        Ok(())
    }
    pub fn end_message(&mut self) -> Result<(), S::Error> {
        self.state = match self.state {
            EncodeState::Header | EncodeState::Data => {
                self.sink.terminate_message()?;
                EncodeState::End
            }
            EncodeState::End => EncodeState::End,
            _ => return Err(EncodeError::InvalidEncodeState(self.state).into()),
        };
        Ok(())
    }
    pub fn finish(mut self) -> Result<S, S::Error> {
        self.end_message()?;
        Ok(self.sink)
    }
    /// Encodes a boolean into program data bytes.
    ///
    /// Reference: SCPI 1999.0: 7.3 - Boolean Program Data
    pub fn encode_boolean(&mut self, value: bool) -> Result<(), S::Error> {
        self.write_byte(match value {
            true => b'1',
            false => b'0',
        })
    }
    /// Encodes a string value into character program data bytes.
    ///
    /// Reference: IEEE 488.2: 7.7.1 - \<CHARACTER PROGRAM DATA\>
    pub fn encode_characters(&mut self, value: &str) -> Result<(), S::Error> {
        if is_program_mnemonic(value.as_bytes()) {
            self.write_bytes(value.as_bytes())
        } else {
            Err(EncodeError::InvalidCharacterData.into())
        }
    }
    /// Encodes an integer value into decimal numeric program data bytes.
    ///
    /// Reference: IEEE 488.2: 7.7.2 - \<DECIMAL NUMERIC PROGRAM DATA\>
    pub fn encode_numeric_integer<T: Integer>(&mut self, value: T) -> Result<(), S::Error> {
        let mut fmt: ArrayBuffer<32> = ArrayBuffer::new();
        let res = write!(&mut fmt, "{}", value);
        debug_assert_eq!(res, Ok(()));
        self.write_bytes(fmt.finish())
    }
    /// Encodes a floating point value into decimal numeric program data bytes.
    ///
    /// References:
    ///   - IEEE 488.2: 7.7.2 - \<DECIMAL NUMERIC PROGRAM DATA\>
    ///   - SCPI 1999.0: 7.2 - Decimal Numeric Program Data
    pub fn encode_numeric_float<T: Float>(&mut self, value: T) -> Result<(), S::Error> {
        // TODO: consider validating the range?
        if value.is_finite() {
            let mut fmt: ArrayBuffer<64> = ArrayBuffer::new();
            let res = write!(&mut fmt, "{:E}", value);
            debug_assert_eq!(res, Ok(()));
            self.write_bytes(fmt.finish())
        } else if value.is_nan() {
            // SCPI 1999.0: 7.2.1.5 - Not A Number (NAN)
            self.write_bytes(b"NAN")
        } else {
            // SCPI 1999.0: 7.2.1.4 - INFinity and Negative INFinity (NINF)
            if value.is_sign_positive() {
                self.write_bytes(b"INF")
            } else {
                self.write_bytes(b"NINF")
            }
        }
    }
    /// Encodes an ASCII string into IEEE 488.2 string program data bytes.
    ///
    /// Reference: IEEE 488.2: 7.7.5 - \<STRING PROGRAM DATA\>
    pub fn encode_string(&mut self, data: &str) -> Result<(), S::Error> {
        if data.as_bytes().iter().all(|ch| ch.is_ascii()) {
            // IEEE 488.2: 7.7.5.2 - Encoding syntax
            self.write_byte(b'"')?;
            let mut chunk_iter = data.as_bytes().split(|&ch| ch == b'"').peekable();
            while let Some(chunk) = chunk_iter.next() {
                self.write_bytes(chunk)?;
                if chunk_iter.peek().is_some() {
                    self.write_bytes(b"\"\"")?;
                }
            }
            self.write_byte(b'"')
        } else {
            Err(EncodeError::NonAsciiString.into())
        }
    }
    /// Encodes a IEEE 488.2 definite length arbitrary block header declaring the given length.
    ///
    /// Reference: IEEE 488.2: 7.7.6 - \<ARBITRARY BLOCK PROGRAM DATA\>
    pub fn encode_definite_block_header(&mut self, len: usize) -> Result<(), S::Error> {
        let mut fmt: ArrayBuffer<11> = ArrayBuffer::new();

        // IEEE 488.2: 7.7.6.2 - Encoding syntax
        write!(&mut fmt, "#0{}", len).map_err(|_| EncodeError::BlockSizeOverflow(len))?;
        let header = fmt.finish();
        let digits = header[2..].len();
        header[1] = b'0' + (digits as u8);
        self.write_bytes(header)
    }
    /// Encodes a slice of bytes into IEEE 488.2 definite length arbitrary block bytes.
    ///
    /// Reference: IEEE 488.2: 7.7.6 - \<ARBITRARY BLOCK PROGRAM DATA\>
    pub fn encode_definite_block(&mut self, data: &[u8]) -> Result<(), S::Error> {
        self.encode_definite_block_header(data.len())?;
        self.write_bytes(data)
    }
}
