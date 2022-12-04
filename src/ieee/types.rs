// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use bitflags::bitflags;

use crate::{
    decode::{DecodeError, Decoder},
    encode::{EncodeSink, Encoder},
    program_data::ProgramData,
    response_data::ResponseData,
    ArbitraryAscii, ByteSource,
};

/// IEEE 488.2 Device identification response
///
/// Returned by Identification Query (*IDN?).
///
/// Reference: IEEE 488.2: 10.14 - *IDN?, Identification Query
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceIdentification {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub firmware_level: String,
}

impl DeviceIdentification {
    pub fn from_response(text: &str) -> Option<Self> {
        let mut iter = text.split(',').map(|field| match field.trim() {
            "0" => String::new(),
            value => value.to_owned(),
        });
        let parts = (
            iter.next(),
            iter.next(),
            iter.next(),
            iter.next(),
            iter.next(),
        );

        match parts {
            (Some(manufacturer), Some(model), Some(serial_number), Some(firmware_level), None) => {
                Some(DeviceIdentification {
                    manufacturer,
                    model,
                    serial_number,
                    firmware_level,
                })
            }
            _ => None,
        }
    }
}

impl ResponseData for DeviceIdentification {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        let text: String = ArbitraryAscii::decode(decoder)?.into();
        DeviceIdentification::from_response(&text).ok_or_else(|| DecodeError::Parse.into())
    }
}

// IEEE 488.2 List of macro labels
//
// Returned by Learn Macro Query (*LMC?).
//
// Reference: IEEE 488.2: 10.16 - *LMC?, Learn Macro Query
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MacroList(pub Vec<String>);

impl ResponseData for MacroList {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        let mut labels = Vec::new();
        let first = String::decode(decoder)?;
        if first.is_empty() {
            Ok(MacroList(labels))
        } else {
            labels.push(first);
            loop {
                labels.push(String::decode(decoder)?);
                if decoder.is_at_end() {
                    break Ok(MacroList(labels));
                }
            }
        }
    }
}

bitflags! {
    /// IEEE 488.2 Standard event status register value
    ///
    /// Reference: IEEE 488.2: 11.5.1 - Standard Event Status Register Model
    pub struct StandardEventStatus: u16 {
        /// Power On
        ///
        /// Reference: IEEE 488.2: 11.5.1.1.2 Bit 7 - Power On (PON)
        const PON = 0b1000_0000;
        /// User Request
        ///
        /// Reference: IEEE 488.2: 11.5.1.1.3 Bit 6 - User Request (URQ)
        const URQ = 0b0100_0000;
        /// Command Error
        ///
        /// Reference: IEEE 488.2: 11.5.1.1.4 Bit 5 - Command ERROR (CME)
        const CME = 0b0010_0000;
        /// Execution Error
        ///
        /// Reference: IEEE 488.2: 11.5.1.1.5 Bit 4 - Execution ERROR (E)
        const E   = 0b0001_0000;
        /// Device-Specific Error
        ///
        /// Reference: IEEE 488.2: 11.5.1.1.6 Bit 3 - Device-Specific ERROR (DDE)
        const DDE = 0b0000_1000;
        /// Query Error
        ///
        /// Reference: IEEE 488.2: 11.5.1.1.7 Bit 2 - Query ERROR (QYE)
        const QYE = 0b0000_0100;
        /// Request Control
        ///
        /// Reference: IEEE 488.2: 11.5.1.1.8 Bit 1 - Request Control (RQC)
        const RQC = 0b0000_0010;
        /// Operation Complete
        ///
        /// Reference: IEEE 488.2: 11.5.1.1.9 Bit 0 - Operation Complete (OPC)
        const OPC = 0b0000_0001;
    }
}

impl ProgramData for StandardEventStatus {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        self.bits().encode(encoder)
    }
}

impl ResponseData for StandardEventStatus {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        let value = u16::decode(decoder)?;
        StandardEventStatus::from_bits(value).ok_or_else(|| DecodeError::Parse.into())
    }
}

/// IEEE 488.2 Status Byte Register
///
/// Reference: IEEE 488.2: 11.2 - Status Byte Register
pub type StatusByte = u8;
