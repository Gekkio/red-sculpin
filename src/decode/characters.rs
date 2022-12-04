// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt;

use super::{upper, Decoder};
use crate::{decode::DecodeError, ByteSource};

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

#[cfg(test)]
mod tests {
    use crate::{decode::Decoder, Error};

    #[test]
    fn uppercase_and_underscores_are_valid() {
        match decode(b"AS_DF123\n").as_deref() {
            Ok("AS_DF123") => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn lowercase_is_invalid() {
        match decode(b"nope\n") {
            Err(Error::Decode(_)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn other_characters_are_invalid() {
        match decode(b"FAIL!") {
            Err(Error::Decode(_)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
        match decode("FAIL€€".as_bytes()) {
            Err(Error::Decode(_)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn data_cant_be_empty() {
        match decode(b"\n") {
            Err(Error::Decode(_)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    fn decode(bytes: &'static [u8]) -> Result<String, Error> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        let mut buffer = String::new();
        super::decode_characters(&mut decoder, &mut buffer)?;
        Ok(buffer)
    }
}
