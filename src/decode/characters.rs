// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt;

use super::Decoder;
use crate::{decode::DecodeError, ByteSource};

/// Decodes character response data
///
/// Reference: IEEE 488.2: 8.7.1 - \<CHARACTER RESPONSE DATA\>
impl<S: ByteSource> Decoder<S> {
    pub fn decode_characters<T: fmt::Write>(&mut self, target: &mut T) -> Result<(), S::Error> {
        target
            .write_char(self.upper()? as char)
            .map_err(|_| DecodeError::BufferOverflow)?;
        loop {
            match self.read_byte()? {
                byte @ b'A'..=b'Z' | byte @ b'0'..=b'9' | byte @ b'_' => target
                    .write_char(byte as char)
                    .map_err(|_| DecodeError::BufferOverflow)?,
                byte => break self.end_with(byte),
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
    fn uppercase_and_underscores_are_valid() {
        assert_matches!(decode(b"AS_DF123\n").as_deref(), Ok("AS_DF123"));
    }

    #[test]
    fn lowercase_is_invalid() {
        assert_matches!(decode(b"nope\n"), Err(DecodeError::Parse));
    }

    #[test]
    fn other_characters_are_invalid() {
        assert_matches!(
            decode(b"FAIL!"),
            Err(DecodeError::InvalidDataTerminator { byte: b'!' })
        );
        assert_matches!(
            decode("FAIL€€".as_bytes()),
            Err(DecodeError::InvalidDataTerminator { byte: 0xe2 })
        );
    }

    #[test]
    fn data_cant_be_empty() {
        assert_matches!(decode(b"\n"), Err(DecodeError::Parse));
    }

    fn decode(bytes: &'static [u8]) -> Result<String, DecodeError> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        let mut buffer = String::new();
        decoder.decode_characters(&mut buffer)?;
        Ok(buffer)
    }
}
