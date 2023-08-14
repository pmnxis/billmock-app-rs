/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#[cfg(test)] // for the print out internnal log in test code
extern crate std;

use crc::{Crc, CRC_32_ISO_HDLC};
use serial_arcade_pay::*;

// Common initial header src and format
pub(crate) const HEADER_SRC_CARD_READER: u8 = 0xCA;
pub(crate) const HEADER_SRC_BILLMOCK: u8 = 0xFE;
pub(crate) const HEADER_FORMAT_BYTE: u8 = 0x01;
pub(crate) const HEADER_FORMAT_STR: u8 = 0x02;

pub(crate) const RAW_DATA_SRC_OFFSET: usize = 0;
pub(crate) const RAW_DATA_FORMAT_OFFSET: usize = 1;
pub(crate) const RAW_DATA_LEN_H_OFFSET: usize = 2;
pub(crate) const RAW_DATA_LEN_L_OFFSET: usize = 3;
pub(crate) const RAW_DATA_CRC32_OFFSET: usize = 4;
pub(crate) const RAW_DATA_TEXT_OFFSET: usize = 8;

/// warn! for no_std , std(test)
/// Work as `defmt::warn!` on no_std environment
/// Work as `std::println!` on test coverage with std
#[cfg(not(test))]
macro_rules! hybrid_warn {
    ($($args:tt)*) => {
        #[cfg(not(test))]
        defmt::warn!($($args)*);
    };
}

/// warn! for no_std , std(test)
/// Work as `defmt::warn!` on no_std environment
/// Work as `std::println!` on test coverage with std
#[cfg(test)]
macro_rules! hybrid_warn {
    ($($args:tt)*) => {
        log::warn!($($args)*);
    };
}

#[allow(dead_code)]
pub(crate) fn str_to_u32(s: &[u8]) -> Option<u32> {
    let mut result = 0u32;
    for c in s.iter() {
        let digit = {
            if (b'0' <= *c) && (*c <= b'9') {
                Some(*c - b'0')
            } else {
                None
            }
        }?;
        result = result.checked_mul(10)?.checked_add(digit as u32)?;
    }
    Some(result)
}

// Custom iterator to split tokens
pub(crate) struct TokenSplitter<'a> {
    text: &'a [u8],
    separator: u8,
    current_index: usize,
}

#[allow(dead_code)]
impl<'a> TokenSplitter<'a> {
    fn new(text: &'a [u8], separator: u8) -> TokenSplitter<'a> {
        TokenSplitter {
            text,
            separator,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for TokenSplitter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.text.len() {
            return None;
        }

        let start = self.current_index;
        if let Some(end) = self.text[start..].iter().position(|&c| c == self.separator) {
            self.current_index = start + end + 1;
            Some(&self.text[start..start + end])
        } else {
            self.current_index = self.text.len();
            Some(&self.text[start..])
        }
    }
}

#[allow(dead_code)]
pub(crate) fn split_tokens(text: &[u8], separator: u8) -> TokenSplitter<'_> {
    TokenSplitter::new(text, separator)
}

/// CRC32-ISO3309(HDLC)
fn inner_calc_checksum(raw_data: &[u8]) -> u32 {
    // todo!, utilize STM32 CRC unit.
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let mut digest = crc.digest();
    digest.update(&raw_data);

    digest.finalize()
}

/// Calculate actual checksum from packet data
pub(crate) fn actual_checksum(raw_data: &[u8], raw_len: usize) -> u32 {
    inner_calc_checksum(&raw_data[RAW_DATA_TEXT_OFFSET..raw_len])
}

/// Get expected big-endian u32 checksum from packet
pub(crate) fn expected_checksum(raw_data: &[u8]) -> u32 {
    // big endian
    ((raw_data[RAW_DATA_CRC32_OFFSET] as u32) << 24)
        | ((raw_data[RAW_DATA_CRC32_OFFSET + 1] as u32) << 16)
        | ((raw_data[RAW_DATA_CRC32_OFFSET + 2] as u32) << 8)
        | (raw_data[RAW_DATA_CRC32_OFFSET + 3] as u32)
}

/// Get actual big-endian packet length from the packet data
pub(crate) fn actual_len(raw_data: &[u8]) -> Result<usize, SerialArcadeError> {
    match raw_data.len() < 8 {
        true => Err(SerialArcadeError::InvalidFrame),
        false => Ok(raw_data[RAW_DATA_LEN_L_OFFSET] as usize
            + ((raw_data[RAW_DATA_LEN_H_OFFSET] as usize) << 8)),
    }
}

pub trait AnyExampleDevice: Sized {
    fn test_type(inner_data: &[u8]) -> bool;

    fn parse(inner_data: &[u8]) -> Option<Self>;

    fn request_form(request: &GenericPaymentRequest, tx_inner_buff: &mut [u8]) -> Option<usize>;

    fn degrade(&self) -> GenericPaymentRecv;

    fn self_degrade(self) -> GenericPaymentRecv;
}
