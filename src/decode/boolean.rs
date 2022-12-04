// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Decoder;
use crate::{decode::DecodeError, ByteSource};

/// Decodes boolean response data.
///
/// IEEE 488.2 does not formally specify a response format for booleans, but commands with boolean
/// responses tend to use NR1 numerical literals 0 and 1, which match the SCPI boolean format spec.
///
/// Reference: SCPI 1999.0: 7.3 - Boolean Program Data
impl<S: ByteSource> Decoder<S> {
    pub fn decode_boolean(&mut self) -> Result<bool, S::Error> {
        match self.read_byte()? {
            b'0' => {
                let byte = self.read_byte()?;
                self.end_with(byte)?;
                Ok(false)
            }
            b'1' => {
                let byte = self.read_byte()?;
                self.end_with(byte)?;
                Ok(true)
            }
            _ => Err(DecodeError::Parse.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;

    use crate::decode::{DecodeError, Decoder};

    #[test]
    fn zero_is_false() {
        assert_matches!(decode(b"0\n"), Ok(false));
    }

    #[test]
    fn one_is_true() {
        assert_matches!(decode(b"1\n"), Ok(true));
    }

    #[test]
    fn extra_chars_are_not_allowed() {
        assert_matches!(
            decode(b"10\n"),
            Err(DecodeError::InvalidDataTerminator { byte: b'0' })
        );
    }

    #[test]
    fn textual_forms_are_not_valid() {
        assert_matches!(decode(b"false\n"), Err(DecodeError::Parse));
        assert_matches!(decode(b"true\n"), Err(DecodeError::Parse));
    }

    fn decode(bytes: &'static [u8]) -> Result<bool, DecodeError> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        decoder.decode_boolean()
    }
}
