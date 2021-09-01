// SPDX-FileCopyrightText: 2020-2021 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::encode::{EncodeSink, Encoder};

/// Trait for types that can be used as IEEE/SCPI message program data
pub trait ProgramData {
    /// Encodes this value as bytes into the given encoder.
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error>;
}

/// A homogeneous list of program data values
pub struct ProgramList<'a, T>(pub &'a [T]);

impl<'a, T> ProgramData for ProgramList<'a, T>
where
    T: ProgramData,
{
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        for data in self.0 {
            data.encode(encoder)?;
        }
        Ok(())
    }
}

impl ProgramData for () {
    fn encode<S: EncodeSink>(&self, _: &mut Encoder<S>) -> Result<(), S::Error> {
        Ok(())
    }
}

impl<T> ProgramData for Option<T>
where
    T: ProgramData,
{
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        if let Some(data) = self {
            data.encode(encoder)
        } else {
            Ok(())
        }
    }
}

impl ProgramData for f32 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_float(*self)
    }
}

impl ProgramData for f64 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_float(*self)
    }
}

impl ProgramData for u8 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for u16 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for u32 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for u64 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for u128 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for usize {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}

impl ProgramData for i8 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for i16 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for i32 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for i64 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for i128 {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}
impl ProgramData for isize {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_numeric_integer(*self)
    }
}

impl<'a> ProgramData for &'a str {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_string(*self)
    }
}

impl ProgramData for str {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_string(self)
    }
}

impl<'a> ProgramData for &'a [u8] {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_definite_block(*self)
    }
}

impl ProgramData for [u8] {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_definite_block(self)
    }
}

impl ProgramData for bool {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        encoder.encode_boolean(*self)
    }
}

pub trait Encodable<T> {
    type Error;
    fn encode(&mut self, value: &T) -> Result<(), Self::Error>;
}

impl<A, B> ProgramData for (A, B)
where
    A: ProgramData,
    B: ProgramData,
{
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        self.0.encode(encoder)?;
        self.1.encode(encoder)
    }
}

impl<A, B, C> ProgramData for (A, B, C)
where
    A: ProgramData,
    B: ProgramData,
    C: ProgramData,
{
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        self.0.encode(encoder)?;
        self.1.encode(encoder)?;
        self.2.encode(encoder)
    }
}

impl<A, B, C, D> ProgramData for (A, B, C, D)
where
    A: ProgramData,
    B: ProgramData,
    C: ProgramData,
    D: ProgramData,
{
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        self.0.encode(encoder)?;
        self.1.encode(encoder)?;
        self.2.encode(encoder)?;
        self.3.encode(encoder)
    }
}

#[cfg(test)]
fn encode_test<F: FnOnce(&mut Encoder<Vec<u8>>) -> Result<(), crate::Error>>(
    f: F,
) -> Result<Vec<u8>, crate::Error> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.begin_message_unit()?;
    encoder.write_bytes(b"TEST")?;
    f(&mut encoder)?;
    encoder.end_message()?;
    encoder.finish()
}

#[test]
fn test_str() {
    let result = encode_test(|encoder| "foo".encode(encoder)).unwrap();
    assert_eq!(result, b"TEST \"foo\"\n");
}

#[test]
fn test_str_escape() {
    let result =
        encode_test(|encoder| r#"what if "quotes" break 'stuff'?"#.encode(encoder)).unwrap();
    assert_eq!(result, b"TEST \"what if \"\"quotes\"\" break 'stuff'?\"\n");
}

#[test]
fn test_definite_block() {
    let result = encode_test(|encoder| [0x11, 0x22, 0x33].encode(encoder)).unwrap();
    assert_eq!(result, b"TEST #13\x11\x22\x33\n");
}

#[test]
fn test_f32_positive() {
    let result = encode_test(|encoder| 1.2345678E11f32.encode(encoder)).unwrap();
    assert_eq!(result, b"TEST 1.2345678E11\n");
}

#[test]
fn test_f32_negative() {
    let result = encode_test(|encoder| (-1.2345678E-11f32).encode(encoder)).unwrap();
    assert_eq!(result, b"TEST -1.2345678E-11\n");
}

#[test]
fn test_f64_positive() {
    let result = encode_test(|encoder| 1.234567891234567E11f64.encode(encoder)).unwrap();
    assert_eq!(result, b"TEST 1.234567891234567E11\n");
}

#[test]
fn test_f64_negative() {
    let result = encode_test(|encoder| (-1.234567891234567E-11f64).encode(encoder)).unwrap();
    assert_eq!(result, b"TEST -1.234567891234567E-11\n");
}

#[test]
fn test_tuple2() {
    let result = encode_test(|encoder| ("mixed", -42i32).encode(encoder)).unwrap();
    assert_eq!(result, b"TEST \"mixed\",-42\n");
}

#[test]
fn test_tuple3() {
    let result = encode_test(|encoder| (1u8, -1i8, -420000f32).encode(encoder)).unwrap();
    assert_eq!(result, b"TEST 1,-1,-4.2E5\n");
}

#[test]
fn test_tuple4() {
    let result = encode_test(|encoder| {
        (&[1, 2, 3][..], f64::NAN, f32::INFINITY, f32::NEG_INFINITY).encode(encoder)
    })
    .unwrap();
    assert_eq!(result, b"TEST #13\x01\x02\x03,NAN,INF,NINF\n");
}
