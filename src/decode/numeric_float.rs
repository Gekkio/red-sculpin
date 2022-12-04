// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::string::String;

use super::Decoder;
use crate::{decode::DecodeError, internal::Float, ByteSource};

/// Decodes numeric float response data in plain (NR2) or exponential (NR3) format.
///
/// References:
///
/// - IEEE 488.2: 8.7.3 - \<NR2 NUMERIC RESPONSE DATA\>
/// - IEEE 488.2: 8.7.4 - \<NR3 NUMERIC RESPONSE DATA\>
impl<S: ByteSource> Decoder<S> {
    pub fn decode_numeric_float<T: Float>(&mut self) -> Result<T, S::Error> {
        let mut buf = String::new();
        match self.read_byte()? {
            byte @ b'+' | byte @ b'-' => {
                buf.push(byte as char);
                buf.push(self.digit()? as char);
            }
            byte @ b'0'..=b'9' => buf.push(byte as char),
            _ => return Err(DecodeError::Parse.into()),
        };
        loop {
            match self.read_byte()? {
                byte @ b'0'..=b'9' => buf.push(byte as char),
                byte @ b'.' => break buf.push(byte as char),
                _ => return Err(DecodeError::Parse.into()),
            }
        }
        match self.read_byte()? {
            byte @ b'0'..=b'9' => buf.push(byte as char),
            _ => return Err(DecodeError::Parse.into()),
        }
        loop {
            match self.read_byte()? {
                byte @ b'0'..=b'9' => buf.push(byte as char),
                byte @ b'E' => break buf.push(byte as char),
                byte => {
                    self.end_with(byte)?;
                    return T::from_str(&buf).map_err(|_| DecodeError::Parse.into());
                }
            }
        }
        buf.push(self.sign()? as char);
        buf.push(self.digit()? as char);
        loop {
            match self.read_byte()? {
                byte @ b'0'..=b'9' => buf.push(byte as char),
                byte => {
                    self.end_with(byte)?;
                    break T::from_str(&buf).map_err(|_| DecodeError::Parse.into());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        decode::{DecodeError, Decoder},
        internal::Float,
    };

    mod plain_format {
        use super::decode;

        #[test]
        fn positive_value() {
            let data = b"42.69\n";
            match decode::<f32>(data) {
                Ok(value) if value == 42.69 => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<f64>(data) {
                Ok(value) if value == 42.69 => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn negative_value() {
            let data = b"-5.123456789\n";
            match decode::<f32>(data) {
                Ok(value) if value == -5.123456789 => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<f64>(data) {
                Ok(value) if value == -5.123456789 => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn integer_part_is_mandatory() {
            let data = b".42\n";
            match decode::<f32>(data) {
                Err(_) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn decimal_separator_is_mandatory() {
            let data = b"42\n";
            match decode::<f32>(data) {
                Err(_) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn fractional_part_is_mandatory() {
            let data = b"42.\n";
            match decode::<f32>(data) {
                Err(_) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
    }

    mod exponential_format {
        use super::decode;

        #[test]
        fn positive_exponent() {
            let data = b"1.0005E+3\n";
            match decode::<f32>(data) {
                Ok(value) if value == 1.0005E3 => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<f64>(data) {
                Ok(value) if value == 1.0005E3 => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn negative_exponent() {
            let data = b"-99.123E-1\n";
            match decode::<f32>(data) {
                Ok(value) if value == -99.123E-1 => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<f64>(data) {
                Ok(value) if value == -99.123E-1 => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn exponent_sign_is_mandatory() {
            let data = b"1.0E3\n";
            match decode::<f32>(data) {
                Err(_) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
    }

    fn decode<T: Float>(bytes: &'static [u8]) -> Result<T, DecodeError> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        decoder.decode_numeric_float()
    }
}
