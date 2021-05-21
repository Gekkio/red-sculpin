// SPDX-FileCopyrightText: 2020-2021 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

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

use core::str;

use encode::EncodeError;

use crate::{
    decode::{DecodeError, Decoder},
    encode::Encoder,
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

impl<'a> ByteSource for &'a [u8] {
    type Error = DecodeError;

    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        if self.len() == 0 {
            Err(DecodeError::UnexpectedEnd)
        } else {
            let (l, r) = self.split_at(1);
            *self = r;
            Ok(l[0])
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

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.push(byte);
        Ok(())
    }
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.extend(bytes);
        Ok(())
    }
}

/// Trait for types that represent IEEE/SCPI commands
pub trait Command {
    type ProgramData: ProgramData;
    fn mnemonic(&self) -> &str;
    fn program_data(&self) -> Option<Self::ProgramData>;
    fn encode<S: ByteSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_message_unit()?;
        encoder.write_bytes(self.mnemonic().as_bytes())?;
        if let Some(program_data) = self.program_data() {
            program_data.encode(encoder)?;
        }
        Ok(())
    }
}

/// Trait for types that represent IEEE/SCPI queries
pub trait Query {
    type ProgramData: ProgramData;
    type ResponseData: ResponseData;
    fn mnemonic(&self) -> &str;
    fn program_data(&self) -> Option<Self::ProgramData>;
    fn encode<S: ByteSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_message_unit()?;
        encoder.write_bytes(self.mnemonic().as_bytes())?;
        if let Some(program_data) = self.program_data() {
            program_data.encode(encoder)?;
        }
        Ok(())
    }
    fn decode<S: ByteSource>(
        &self,
        decoder: &mut Decoder<S>,
    ) -> Result<Self::ResponseData, S::Error> {
        Self::ResponseData::decode(decoder)
    }
}

#[macro_export]
macro_rules! declare_tuple_command {
    ($(#[$attr:meta])* pub struct $name:ident<$l:lifetime, $mne:literal>;) => {
        $(#[$attr])*
        pub struct $name<$l>;

        impl<$l> $crate::Command for $name<$l> {
            type ProgramData = ();
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Option<&Self::ProgramData> { None }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$mne:literal>;) => {
        $(#[$attr])*
        pub struct $name;

        impl $crate::Command for $name {
            type ProgramData = ();
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Option<Self::ProgramData> { None }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$l:lifetime, $mne:literal>(pub $prog:ty);) => {
        $(#[$attr])*
        pub struct $name<$l>(pub $prog);

        impl<$l> $crate::Command for $name<$l> {
            type ProgramData = $prog;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Option<Self::ProgramData> { Some(self.0) }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$mne:literal>(pub $prog:ty);) => {
        $(#[$attr])*
        pub struct $name(pub $prog);

        impl $crate::Command for $name {
            type ProgramData = $prog;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Option<Self::ProgramData> { Some(self.0) }
        }
    }
}

#[macro_export]
macro_rules! declare_tuple_query {
    ($(#[$attr:meta])* pub struct $name:ident<$l:lifetime, $mne:literal, $res:ty>;) => {
        $(#[$attr])*
        pub struct $name<$li>;

        impl<$l> $crate::Query for $name<$l> {
            type ProgramData = ();
            type ResponseData = $res;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Option<&Self::ProgramData> { None }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$mne:literal, $res:ty>;) => {
        $(#[$attr])*
        pub struct $name;

        impl $crate::Query for $name {
            type ProgramData = ();
            type ResponseData = $res;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Option<Self::ProgramData> { None }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$l:lifetime, $mne:literal, $res:ty>(pub $prog:ty);) => {
        $(#[$attr])*
        pub struct $name<$l>(pub $prog);

        impl<$l> $crate::Query for $name<$l> {
            type ProgramData = $prog;
            type ResponseData = $res;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Option<Self::ProgramData> { Some(self.0) }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$mne:literal, $res:ty>(pub $prog:ty);) => {
        $(#[$attr])*
        pub struct $name(pub $prog);

        impl $crate::Query for $name {
            type ProgramData = $prog;
            type ResponseData = $res;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Option<Self::ProgramData> { Some(self.0) }
        }
    }
}
