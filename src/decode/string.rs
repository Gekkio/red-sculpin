// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt;

use super::Decoder;
use crate::{decode::DecodeError, ByteSource};

/// Decodes string response data into the given target buffer.
///
/// As per IEEE 488.2, only ASCII is supported.
///
/// Reference: IEEE 488.2: 8.7.8 - \<STRING RESPONSE DATA\>
impl<S: ByteSource> Decoder<S> {
    pub fn decode_string<T: fmt::Write>(&mut self, target: &mut T) -> Result<(), S::Error> {
        self.quote()?;
        loop {
            match self.read_byte()? {
                b'"' => match self.read_byte()? {
                    b'"' => target
                        .write_char('"')
                        .map_err(|_| DecodeError::BufferOverflow)?,
                    byte => break self.end_with(byte),
                },
                byte if byte.is_ascii() => target
                    .write_char(byte as char)
                    .map_err(|_| DecodeError::BufferOverflow)?,
                _ => break Err(DecodeError::Parse.into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use matches::assert_matches;

    use crate::decode::{DecodeError, Decoder};

    #[test]
    fn data_must_be_quoted() {
        assert_matches!(decode(b"\"Quoted\"\n").as_deref(), Ok("Quoted"));
        assert_matches!(decode(b"notquoted\n").as_deref(), Err(DecodeError::Parse));
    }

    #[test]
    fn opening_quote_is_mandatory() {
        assert_matches!(decode(b"Invalid\"\n").as_deref(), Err(DecodeError::Parse));
    }

    #[test]
    fn closing_quote_is_mandatory() {
        assert_matches!(
            decode(b"\"Invalid\n").as_deref(),
            Err(DecodeError::UnexpectedEnd)
        );
    }

    #[test]
    fn quotes_are_escaped_by_doubling() {
        assert_matches!(
            decode(b"\"quote:\"\"\"\n").as_deref(),
            Ok("quote:\"")
        );
    }

    fn decode(bytes: &'static [u8]) -> Result<String, DecodeError> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        let mut buffer = String::new();
        decoder.decode_string(&mut buffer)?;
        Ok(buffer)
    }
}
