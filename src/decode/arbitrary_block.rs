// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::str;

use super::Decoder;
use crate::{decode::DecodeError, internal::ArrayBuffer, ByteSink, ByteSource};

/// Decodes arbitrary block response data into the given target buffer.
///
/// References:
///
/// - IEEE 488.2: 8.7.9 - \<DEFINITE LENGTH ARBITRARY BLOCK RESPONSE DATA\>
/// - IEEE 488.2: 8.7.10 - \<INDEFINITE LENGTH ARBITRARY BLOCK RESPONSE DATA\>
impl<S: ByteSource> Decoder<S> {
    pub fn decode_arbitrary_block<T: ByteSink>(&mut self, target: &mut T) -> Result<(), S::Error> {
        match self.read_byte()? {
            b'#' => (),
            _ => return Err(DecodeError::Parse.into()),
        }
        match self.read_byte()? {
            byte @ b'1'..=b'9' => {
                // definite length format
                let digits = (byte - b'0') as usize;
                let mut buf = ArrayBuffer::<9>::new();
                for _ in 0..digits {
                    buf.push(self.digit()?)
                        .map_err(|_| DecodeError::BufferOverflow)?;
                }
                let block_size = str::from_utf8(buf.finish())
                    .ok()
                    .and_then(|text| text.parse().ok())
                    .ok_or(DecodeError::Parse)?;
                for _ in 0..block_size {
                    target
                        .write_byte(self.read_byte()?)
                        .map_err(|_| DecodeError::BufferOverflow)?;
                }
                let byte = self.read_byte()?;
                self.end_with(byte)
            }
            b'0' => loop {
                // indefinite length format
                match self.read_byte()? {
                    byte @ b'\n' => break self.end_with(byte),
                    byte => target
                        .write_byte(byte)
                        .map_err(|_| DecodeError::BufferOverflow)?,
                }
            },
            _ => Err(DecodeError::Parse.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{decode::Decoder, Error};

    #[test]
    fn header_must_exist() {
        match decode(b"\n") {
            Err(Error::Decode(_)) => (),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    mod definite_format {
        use super::decode;
        use crate::Error;

        #[test]
        fn data_can_be_empty() {
            match decode(b"#10\n").as_deref() {
                Ok(b"") => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn header_length_can_be_1() {
            match decode(b"#15short\n").as_deref() {
                Ok(b"short") => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn header_length_can_be_2() {
            match decode(b"#215verylongmessage\n").as_deref() {
                Ok(b"verylongmessage") => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn having_too_few_bytes_leads_to_error() {
            match decode(b"#210truncated\n") {
                Err(Error::Io(_)) => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
    }

    mod indefinite_format {
        use super::decode;

        #[test]
        fn format_uses_terminator_instead_of_length() {
            match decode(b"#0justsomedata\n").as_deref() {
                Ok(b"justsomedata") => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn data_can_be_empty() {
            match decode(b"#0\n").as_deref() {
                Ok(b"") => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }

        #[test]
        fn first_newline_terminates_the_block() {
            match decode("#0Parsed\nNot parsed".as_bytes()).as_deref() {
                Ok(b"Parsed") => (),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
    }

    fn decode(bytes: &'static [u8]) -> Result<Vec<u8>, Error> {
        let mut decoder = Decoder::new(bytes);
        decoder.begin_response_data()?;
        let mut result = Vec::new();
        decoder.decode_arbitrary_block(&mut result)?;
        Ok(result)
    }
}
