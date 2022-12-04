// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::{
    fmt,
    num::{ParseFloatError, ParseIntError},
    str,
};

macro_rules! declare_tuple_command {
    ($(#[$attr:meta])* pub struct $name:ident<$l:lifetime, $mne:literal>;) => {
        $(#[$attr])*
        pub struct $name<$l>;

        impl<$l> $crate::Command for $name<$l> {
            type ProgramData = ();
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Self::ProgramData { }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$mne:literal>;) => {
        $(#[$attr])*
        pub struct $name;

        impl $crate::Command for $name {
            type ProgramData = ();
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Self::ProgramData { }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$l:lifetime, $mne:literal>(pub $prog:ty);) => {
        $(#[$attr])*
        pub struct $name<$l>(pub $prog);

        impl<$l> $crate::Command for $name<$l> {
            type ProgramData = $prog;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Self::ProgramData { self.0 }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$mne:literal>(pub $prog:ty);) => {
        $(#[$attr])*
        pub struct $name(pub $prog);

        impl $crate::Command for $name {
            type ProgramData = $prog;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Self::ProgramData { self.0 }
        }
    }
}

macro_rules! declare_tuple_query {
    ($(#[$attr:meta])* pub struct $name:ident<$l:lifetime, $mne:literal, $res:ty>;) => {
        $(#[$attr])*
        pub struct $name<$li>;

        impl<$l> $crate::Query for $name<$l> {
            type ProgramData = ();
            type ResponseData = $res;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Self::ProgramData { }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$mne:literal, $res:ty>;) => {
        $(#[$attr])*
        pub struct $name;

        impl $crate::Query for $name {
            type ProgramData = ();
            type ResponseData = $res;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Self::ProgramData { }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$l:lifetime, $mne:literal, $res:ty>(pub $prog:ty);) => {
        $(#[$attr])*
        pub struct $name<$l>(pub $prog);

        impl<$l> $crate::Query for $name<$l> {
            type ProgramData = $prog;
            type ResponseData = $res;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Self::ProgramData { self.0 }
        }
    };
    ($(#[$attr:meta])* pub struct $name:ident<$mne:literal, $res:ty>(pub $prog:ty);) => {
        $(#[$attr])*
        pub struct $name(pub $prog);

        impl $crate::Query for $name {
            type ProgramData = $prog;
            type ResponseData = $res;
            fn mnemonic(&self) -> &str { $mne }
            fn program_data(&self) -> Self::ProgramData { self.0 }
        }
    }
}

pub(crate) use declare_tuple_command;
pub(crate) use declare_tuple_query;

pub struct ArrayBuffer<const LEN: usize> {
    buffer: [u8; LEN],
    written: usize,
}

#[derive(Debug)]
pub struct ArrayBufferFull;

impl<const LEN: usize> ArrayBuffer<LEN> {
    pub fn new() -> ArrayBuffer<LEN> {
        ArrayBuffer {
            buffer: [0; LEN],
            written: 0,
        }
    }
    pub fn push(&mut self, byte: u8) -> Result<(), ArrayBufferFull> {
        if self.written < self.buffer.len() {
            self.buffer[self.written] = byte;
            self.written += 1;
            Ok(())
        } else {
            Err(ArrayBufferFull)
        }
    }
    pub fn push_all(&mut self, bytes: &[u8]) -> Result<(), ArrayBufferFull> {
        if self.written + bytes.len() <= self.buffer.len() {
            self.buffer[self.written..(self.written + bytes.len())].copy_from_slice(bytes);
            self.written += bytes.len();
            Ok(())
        } else {
            Err(ArrayBufferFull)
        }
    }
    pub fn finish(&mut self) -> &mut [u8] {
        &mut self.buffer[0..self.written]
    }
}

impl<const LEN: usize> fmt::Write for ArrayBuffer<LEN> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        self.push_all(bytes).map_err(|_| fmt::Error)
    }
}

pub trait Integer: Sized + Copy + Default + fmt::Display {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError>;
}

impl Integer for u8 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        u8::from_str_radix(s, radix)
    }
}

impl Integer for u16 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        u16::from_str_radix(s, radix)
    }
}

impl Integer for u32 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        u32::from_str_radix(s, radix)
    }
}

impl Integer for u64 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        u64::from_str_radix(s, radix)
    }
}

impl Integer for u128 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        u128::from_str_radix(s, radix)
    }
}

impl Integer for usize {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        usize::from_str_radix(s, radix)
    }
}

impl Integer for i8 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        i8::from_str_radix(s, radix)
    }
}

impl Integer for i16 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        i16::from_str_radix(s, radix)
    }
}

impl Integer for i32 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        i32::from_str_radix(s, radix)
    }
}

impl Integer for i64 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        i64::from_str_radix(s, radix)
    }
}

impl Integer for i128 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        i128::from_str_radix(s, radix)
    }
}

impl Integer for isize {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        isize::from_str_radix(s, radix)
    }
}

pub trait Float: Sized + Copy + Default + fmt::UpperExp {
    fn from_str(s: &str) -> Result<Self, ParseFloatError>;
    fn from_str_radix(s: &str, radix: u32) -> Option<Self>;

    fn is_finite(self) -> bool;
    fn is_nan(self) -> bool;
    fn is_sign_positive(self) -> bool;
}

impl Float for f32 {
    #[allow(clippy::float_cmp)]
    fn from_str(s: &str) -> Result<Self, ParseFloatError> {
        let value = core::str::FromStr::from_str(s)?;
        Ok(if value == 9.9E+37 {
            f32::INFINITY
        } else if value == -9.9E+37 {
            f32::NEG_INFINITY
        } else if value == 9.91E+37 {
            f32::NAN
        } else {
            value
        })
    }

    fn from_str_radix(s: &str, radix: u32) -> Option<Self> {
        let int = u32::from_str_radix(s, radix).ok()?;
        Some(int as f32)
    }

    fn is_finite(self) -> bool {
        self.is_finite()
    }

    fn is_nan(self) -> bool {
        self.is_nan()
    }

    fn is_sign_positive(self) -> bool {
        self.is_sign_positive()
    }
}

impl Float for f64 {
    #[allow(clippy::float_cmp)]
    fn from_str(s: &str) -> Result<Self, ParseFloatError> {
        let value = core::str::FromStr::from_str(s)?;
        Ok(if value == 9.9E+37 {
            f64::INFINITY
        } else if value == -9.9E+37 {
            f64::NEG_INFINITY
        } else if value == 9.91E+37 {
            f64::NAN
        } else {
            value
        })
    }

    fn from_str_radix(s: &str, radix: u32) -> Option<Self> {
        let int = u64::from_str_radix(s, radix).ok()?;
        Some(int as f64)
    }

    fn is_finite(self) -> bool {
        self.is_finite()
    }

    fn is_nan(self) -> bool {
        self.is_nan()
    }

    fn is_sign_positive(self) -> bool {
        self.is_sign_positive()
    }
}
