// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt;

use super::Decoder;
use crate::{decode::DecodeError, ByteSource};

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

#[cfg(test)]
mod tests {
    use crate::{decode::Decoder, Error};

    #[test]
    fn data_with_only_terminator_is_an_empty_string() {
        match decode(b"\n").as_deref() {
            Ok("") => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn ascii_string_is_valid() {
        match decode(b"This is ASCII! 123\0\r@# \t \n").as_deref() {
            Ok("This is ASCII! 123\0\r@# \t ") => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn non_ascii_is_not_valid() {
        match decode("This is *not* ASCII: €€!\n".as_bytes()) {
            Err(Error::Decode(_)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn first_newline_terminates_the_string() {
        match decode("Parsed\nNot parsed".as_bytes()).as_deref() {
            Ok("Parsed") => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    fn decode(bytes: &'static [u8]) -> Result<String, Error> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        let mut buffer = String::new();
        super::decode_arbitrary_ascii(&mut decoder, &mut buffer)?;
        Ok(buffer)
    }
}
