/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_std]

#[cfg(test)] // for the print out internnal log in test code
extern crate std;

#[cfg(test)]
mod tests;

#[macro_use]
mod common;
mod example_byte;
mod example_str;
// extern crate serial_arcade_pay;

use common::*;
use example_byte::*;
// use example_str::*;
use serial_arcade_pay::*;

#[derive(Debug, defmt::Format, Clone, Eq, PartialEq)]
pub enum SerialPayVarient {
    /// Byte-ish serial protocol spec
    ExampleByte,
    /// String-ish key-value style serial protocol spec
    ExampleString,
    /// Unknown
    Common,
}

impl SerialArcadePay for SerialPayVarient {
    fn parse_rx(
        raw_data: &[u8],
        raw_len: usize,
    ) -> Result<(GenericPaymentRecv, SerialPayVarient), SerialArcadeError> {
        let varient = match (
            raw_data[RAW_DATA_SRC_OFFSET],
            raw_data[RAW_DATA_FORMAT_OFFSET],
        ) {
            (HEADER_SRC_CARD_READER, HEADER_FORMAT_BYTE) => Ok(SerialPayVarient::ExampleByte),
            (HEADER_SRC_CARD_READER, HEADER_FORMAT_STR) => Ok(SerialPayVarient::ExampleString),
            (HEADER_SRC_CARD_READER, _) => Err(SerialArcadeError::VarientNotSupportRequest),
            (HEADER_SRC_BILLMOCK, _) => Err(SerialArcadeError::WrongSource),
            _ => Err(SerialArcadeError::InvalidFrame),
        }?;

        // length = start from LEN field, end to DATA.
        let length = actual_len(raw_data)?;
        let actual_sliced_raw_len = RAW_DATA_TEXT_OFFSET + length;

        if actual_sliced_raw_len != raw_len {
            hybrid_warn!(
                "Bad length, expected : {}, actual : {}",
                raw_len,
                actual_sliced_raw_len
            );
            return Err(SerialArcadeError::BadLength);
        }

        let actual_checksum = actual_checksum(raw_data, raw_len);
        let expected_checksum = expected_checksum(raw_data);

        if actual_checksum != expected_checksum {
            hybrid_warn!(
                "Different Checksum, expected : 0x{:08X}, actual : 0x{:08X}",
                expected_checksum,
                actual_checksum
            );

            return Err(SerialArcadeError::BadChecksum);
        }

        let inner_data = &raw_data[RAW_DATA_TEXT_OFFSET..raw_len];

        match varient {
            SerialPayVarient::ExampleByte => ExampleByteRecv::parse(inner_data)
                .map_or(Err(SerialArcadeError::UnsupportedSpec), |x| {
                    Ok((x.self_degrade(), SerialPayVarient::ExampleByte))
                }),
            _ => Err(SerialArcadeError::UnsupportedSpec),
        }
    }

    fn generate_tx(
        &self,
        _request: &GenericPaymentRequest,
        _tx_raw_buff: &mut [u8],
    ) -> Result<usize, SerialArcadeError> {
        unimplemented!()
    }

    fn is_nda() -> bool {
        // `serial-arcade-example` is not nda
        false
    }
}
