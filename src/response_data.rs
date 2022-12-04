// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::{string::String, vec::Vec};

use crate::{
    decode::{DecodeError, Decoder},
    ByteSource,
};

/// Trait for types that can be parsed from IEEE/SCPI response bytes
pub trait ResponseData: Sized {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error>;
}

impl ResponseData for bool {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_boolean()
    }
}

impl ResponseData for u8 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}
impl ResponseData for u16 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}
impl ResponseData for u32 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}
impl ResponseData for u64 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}
impl ResponseData for usize {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}

impl ResponseData for i8 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}
impl ResponseData for i16 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}
impl ResponseData for i32 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}
impl ResponseData for i64 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}
impl ResponseData for isize {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_integer()
    }
}

impl ResponseData for f32 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_float()
    }
}

impl ResponseData for f64 {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        decoder.decode_numeric_float()
    }
}

impl ResponseData for String {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        let mut text = String::new();
        decoder.decode_string(&mut text)?;
        Ok(text)
    }
}

impl ResponseData for Vec<u8> {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        let mut result = Vec::new();
        decoder.decode_arbitrary_block(&mut result)?;
        Ok(result)
    }
}

impl<A, B> ResponseData for (A, B)
where
    A: ResponseData,
    B: ResponseData,
{
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        let a = A::decode(decoder)?;
        let b = B::decode(decoder)?;
        Ok((a, b))
    }
}

impl<A, B, C> ResponseData for (A, B, C)
where
    A: ResponseData,
    B: ResponseData,
    C: ResponseData,
{
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        let a = A::decode(decoder)?;
        let b = B::decode(decoder)?;
        let c = C::decode(decoder)?;
        Ok((a, b, c))
    }
}

impl<A, B, C, D> ResponseData for (A, B, C, D)
where
    A: ResponseData,
    B: ResponseData,
    C: ResponseData,
    D: ResponseData,
{
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        let a = A::decode(decoder)?;
        let b = B::decode(decoder)?;
        let c = C::decode(decoder)?;
        let d = D::decode(decoder)?;
        Ok((a, b, c, d))
    }
}

/// IEEE 488.2 Arbitrary Ascii Response Data
///
/// Reference: IEEE 488.2: 8.7.11 - \<ARBITRARY ASCII RESPONSE DATA\>
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitraryAscii(String);

impl From<ArbitraryAscii> for String {
    fn from(ascii: ArbitraryAscii) -> String {
        ascii.0
    }
}

impl ResponseData for ArbitraryAscii {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        let mut text = String::new();
        decoder.decode_arbitrary_ascii(&mut text)?;
        Ok(ArbitraryAscii(text))
    }
}

/// A homogeneous list of response data values
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponseList<T>(pub Vec<T>);

impl<T> ResponseData for ResponseList<T>
where
    T: ResponseData,
{
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        let mut result = Vec::new();
        loop {
            result.push(T::decode(decoder)?);
            if decoder.is_at_end() {
                break Ok(ResponseList(result));
            }
        }
    }
}

/// Trait for types that can be decoded from character response data.
pub trait CharacterResponseData: Sized {
    fn parse(text: &str) -> Option<Self>;
}

impl<T> ResponseData for T
where
    T: CharacterResponseData,
{
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        decoder.begin_response_data()?;
        let mut text = String::new();
        decoder.decode_arbitrary_ascii(&mut text)?;
        T::parse(&text).ok_or_else(|| DecodeError::Parse.into())
    }
}
