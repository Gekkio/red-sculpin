// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{binary_digit, digit, hex_digit, octal_digit, Decoder};
use crate::{decode::DecodeError, internal::Integer, ByteSource};

/// Decodes numeric integer response data in plain (NR1), hexadecimal, octal, or binary format.
///
/// References:
///
/// - IEEE 488.2: 8.7.2 - \<NR1 NUMERIC RESPONSE DATA\>
/// - IEEE 488.2: 8.7.5 - \<HEXADECIMAL NUMERIC RESPONSE DATA\>
/// - IEEE 488.2: 8.7.6 - \<OCTAL NUMERIC RESPONSE DATA\>
/// - IEEE 488.2: 8.7.7 - \<BINARY NUMERIC RESPONSE DATA\>
pub fn decode_numeric_integer<S: ByteSource, T: Integer>(
    decoder: &mut Decoder<S>,
) -> Result<T, S::Error> {
    let mut buf = String::new();
    match decoder.read_byte()? {
        byte @ b'+' | byte @ b'-' => {
            buf.push(byte as char);
            buf.push(digit(decoder)? as char);
        }
        b'#' => match decoder.read_byte()? {
            b'H' => {
                buf.push(hex_digit(decoder)? as char);
                return loop {
                    match decoder.read_byte()? {
                        byte @ b'A'..=b'F' => buf.push(byte as char),
                        byte @ b'0'..=b'9' => buf.push(byte as char),
                        byte => {
                            decoder.end_with(byte)?;
                            break T::from_str_radix(&buf, 16)
                                .map_err(|_| DecodeError::Parse.into());
                        }
                    }
                };
            }
            b'Q' => {
                buf.push(octal_digit(decoder)? as char);
                return loop {
                    match decoder.read_byte()? {
                        byte @ b'0'..=b'7' => buf.push(byte as char),
                        byte => {
                            decoder.end_with(byte)?;
                            break T::from_str_radix(&buf, 8)
                                .map_err(|_| DecodeError::Parse.into());
                        }
                    }
                };
            }
            b'B' => {
                buf.push(binary_digit(decoder)? as char);
                return loop {
                    match decoder.read_byte()? {
                        byte @ b'0' | byte @ b'1' => buf.push(byte as char),
                        byte => {
                            decoder.end_with(byte)?;
                            break T::from_str_radix(&buf, 2)
                                .map_err(|_| DecodeError::Parse.into());
                        }
                    }
                };
            }
            _ => return Err(DecodeError::Parse)?,
        },
        byte @ b'0'..=b'9' => buf.push(byte as char),
        _ => return Err(DecodeError::Parse)?,
    }
    loop {
        match decoder.read_byte()? {
            byte @ b'0'..=b'9' => buf.push(byte as char),
            byte => {
                decoder.end_with(byte)?;
                break T::from_str_radix(&buf, 10).map_err(|_| DecodeError::Parse.into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{decode::Decoder, internal::Integer, Error};

    mod plain_format {
        use super::decode;
        use crate::Error;

        #[test]
        fn positive_value() {
            let data = b"42\n";
            match decode::<u8>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<u32>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<usize>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i32>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<isize>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn negative_value() {
            let data = b"-42\n";
            match decode::<i8>(data) {
                Ok(-42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i32>(data) {
                Ok(-42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<isize>(data) {
                Ok(-42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn unsigned_types_cant_be_negative() {
            match decode::<u8>(b"-42\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn overflow_leads_to_an_error() {
            match decode::<u8>(b"256\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(b"128\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(b"-129\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
    }

    mod hexadecimal_format {
        use super::decode;
        use crate::Error;

        #[test]
        fn positive_value() {
            let data = b"#H2A\n";
            match decode::<u8>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<u32>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<usize>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i32>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<isize>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn negative_values_are_not_supported() {
            match decode::<i8>(b"-#H2A\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(b"#H-2A\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
    }

    mod octal_format {
        use super::decode;
        use crate::Error;

        #[test]
        fn positive_value() {
            let data = b"#Q52\n";
            match decode::<u8>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<u32>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<usize>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i32>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<isize>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn negative_values_are_not_supported() {
            match decode::<i8>(b"-#Q52\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(b"#Q-52\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
    }

    mod binary_format {
        use super::decode;
        use crate::Error;

        #[test]
        fn positive_value() {
            let data = b"#B101010\n";
            match decode::<u8>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<u32>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<usize>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i32>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<isize>(data) {
                Ok(42) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn negative_values_are_not_supported() {
            match decode::<i8>(b"-#B101010\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
            match decode::<i8>(b"#B-101010\n") {
                Err(Error::Decode(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
    }

    #[test]
    fn format_switch_in_middle_is_invalid() {
        match decode::<u8>(b"12#H2A\n") {
            Err(Error::Decode(_)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    fn decode<T: Integer>(bytes: &'static [u8]) -> Result<T, Error> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        super::decode_numeric_integer(&mut decoder)
    }
}
