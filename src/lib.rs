// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]

//! This crate provides a library containing types and low-level protocol functions for controlling
//! IEEE 488.2 / SCPI 1999.0 -compliant test automation hardware
//!
//! Encoding formats:
//!
//! * `u8`/`u16`/`u32`/`u64`/`u128`/`usize`: IEEE 488.2 decimal numeric program data, integer
//! * `i8`/`i16`/`i32`/`i64`/`i128`/`isize`: IEEE 488.2 decimal numeric program data, integer
//! * `f32`/`f64`: IEEE 488.2 decimal numeric program data, exponential format. NaN/Inf/-Inf values
//!   are encoded as character program data (as defined by SCPI)
//! * `&[u8]`: IEEE 488.2 arbitrary block program data, definite length format
//! * `&str`: IEEE 488.2 string program data
//! * `Option<T>`: `Some(value)`=contained value encoded normally, `None`=no value encoded
//! * `ProgramChars`: IEEE 488.2 character program data
//! * `ProgramList`: elements encoded as separate comma-delimited program data values
//!
//! Decoding formats:
//!
//! * `u8`/`u16`/`u32`/`u64`/`u128`/`usize`: IEEE 488.2 numeric response data, all integer formats
//!   are accepted (NR1, hex, oct, bin)
//! * `i8`/`i16`/`i32`/`i64`/`i128`/`isize`: IEEE 488.2 numeric response data, all integer formats
//!   are accepted (NR1, hex, oct, bin)
//! * `f32`/`f64`: IEEE 488.2 numeric response data (NR2/NR3). NaN/Inf/-Inf are interpreted using
//!    IEEE 488.2 recommendations
//! * `Vec<u8>`: IEEE 488.2 arbitrary block response data, both definite and indefinite length
//!   formats are accepted
//! * `String`: IEEE 488.2 string response data
//! * `ArbitraryAscii`: IEEE 488.2 arbitrary ascii response data
//! * `ResponseList`: elements parsed from separate comma-delimited response data values
//!
//! Examples:
//!
//! ```
//! use red_sculpin::{decode::Decoder, encode::Encoder, scpi, Query};
//! use std::net::TcpStream;
//!
//! fn query_system_version(stream: &mut TcpStream) -> Result<f32, red_sculpin::Error> {
//!     let query = scpi::message::SystemVersionQuery; // :SYST:VERS?
//!
//!     let mut encoder = Encoder::new(red_sculpin::Io(stream));
//!     query.encode(&mut encoder)?;
//!     encoder.finish()?;
//!
//!     let mut decoder = Decoder::new(red_sculpin::Io(stream));
//!     let result = query.decode(&mut decoder)?;
//!     decoder.finish()?;
//!     Ok(result)
//! }
//! ```

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::vec::Vec;
use core::str;

use crate::{
    decode::{DecodeError, Decoder},
    encode::{EncodeError, EncodeSink, Encoder},
};
pub use crate::{
    ieee::types::*,
    program_data::{ProgramData, ProgramList},
    response_data::{ArbitraryAscii, CharacterResponseData, ResponseData, ResponseList},
    scpi::types::*,
    utils::is_program_mnemonic,
};

/// Low-level IEEE/SCPI response message decoding
pub mod decode;
/// Low-level IEEE/SCPI program message encoding
pub mod encode;
/// IEEE 488.2 standard
pub mod ieee;
mod internal;
mod program_data;
mod response_data;
/// SCPI 1999.0 standard
pub mod scpi;
mod utils;

/// A source of bytes
pub trait ByteSource {
    type Error: From<DecodeError>;
    fn read_byte(&mut self) -> Result<u8, Self::Error>;
}

impl ByteSource for &[u8] {
    type Error = DecodeError;

    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        match self {
            [first, rest @ ..] => {
                *self = rest;
                Ok(*first)
            }
            [] => Err(DecodeError::UnexpectedEnd),
        }
    }
}

/// A sink for bytes
pub trait ByteSink {
    type Error: From<EncodeError>;
    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.write_bytes(&[byte])
    }
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

impl ByteSink for Vec<u8> {
    type Error = EncodeError;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.extend(bytes);
        Ok(())
    }
}

impl EncodeSink for Vec<u8> {}

/// Trait for types that represent IEEE/SCPI commands
pub trait Command {
    type ProgramData: ProgramData;
    fn mnemonic(&self) -> &str;
    fn program_data(&self) -> Self::ProgramData;
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_message_unit()?;
        encoder.write_bytes(self.mnemonic().as_bytes())?;
        self.program_data().encode(encoder)?;
        Ok(())
    }
}

/// Trait for types that represent IEEE/SCPI queries
pub trait Query {
    type ProgramData: ProgramData;
    type ResponseData: ResponseData;
    fn mnemonic(&self) -> &str;
    fn program_data(&self) -> Self::ProgramData;
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_message_unit()?;
        encoder.write_bytes(self.mnemonic().as_bytes())?;
        self.program_data().encode(encoder)?;
        Ok(())
    }
    fn decode<S: ByteSource>(
        &self,
        decoder: &mut Decoder<S>,
    ) -> Result<Self::ResponseData, S::Error> {
        Self::ResponseData::decode(decoder)
    }
}

#[cfg(feature = "std")]
pub use std_support::*;

#[cfg(feature = "std")]
mod std_support {
    use core::fmt;
    use std::io;

    use super::{ByteSink, ByteSource};
    use crate::{
        decode::DecodeError,
        encode::{EncodeError, EncodeSink},
    };

    pub struct Io<'a, T>(pub &'a mut T);

    impl<'a, T> ByteSource for Io<'a, T>
    where
        T: io::Read,
    {
        type Error = Error;

        fn read_byte(&mut self) -> Result<u8, Self::Error> {
            let mut buf = [0];
            self.0.read_exact(&mut buf)?;
            Ok(buf[0])
        }
    }

    impl<'a, T> ByteSink for Io<'a, T>
    where
        T: io::Write,
    {
        type Error = Error;

        fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
            self.0.write_all(bytes)?;
            Ok(())
        }
    }

    impl<'a, T> EncodeSink for Io<'a, T> where T: io::Write {}

    #[derive(Debug)]
    pub enum Error {
        Encode(EncodeError),
        Decode(DecodeError),
        Io(io::Error),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::Encode(err) => fmt::Display::fmt(err, f),
                Error::Decode(err) => fmt::Display::fmt(err, f),
                Error::Io(err) => fmt::Display::fmt(err, f),
            }
        }
    }

    impl From<EncodeError> for Error {
        fn from(err: EncodeError) -> Self {
            Error::Encode(err)
        }
    }

    impl From<DecodeError> for Error {
        fn from(err: DecodeError) -> Self {
            Error::Decode(err)
        }
    }

    impl From<io::Error> for Error {
        fn from(err: io::Error) -> Self {
            Error::Io(err)
        }
    }

    impl std::error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Error::Encode(err) => Some(err),
                Error::Decode(err) => Some(err),
                Error::Io(err) => Some(err),
            }
        }
    }
}
