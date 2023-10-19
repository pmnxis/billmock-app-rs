/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_std]
#![feature(const_trait_impl)]

#[cfg(test)] // for the print out internnal log in test code
extern crate std;

#[allow(unused)]
mod common;

use card_terminal_adapter::types::*;
use card_terminal_adapter::*;

// #[cfg(any(build, test))]
// mod helper;

pub struct KiccEd785Plug {}

impl CardTerminalConst for KiccEd785Plug {
    fn is_nda() -> bool {
        false
    }
}

impl CardTerminalRxParse for KiccEd785Plug {
    fn pre_parse_common(&self, _raw: &[u8]) -> Result<CardTerminalRxCmd, CardTerminalError> {
        // implement me for actual usage
        Err(CardTerminalError::UnsupportedSpec)
    }

    fn post_parse_response_sale_slot_info(
        &self,
        _raw: &[u8],
    ) -> Result<CardReaderPortBackup, CardTerminalError> {
        // implement me for actual usage
        Err(CardTerminalError::UnsupportedSpec)
    }

    fn post_parse_response_terminal_info(
        &self,
        _raw: &[u8],
    ) -> Result<(TerminalVersion, RawTerminalId), CardTerminalError> {
        // implement me for actual usage
        Err(CardTerminalError::UnsupportedSpec)
    }
}

impl CardTerminalTxGen for KiccEd785Plug {
    fn response_ack<'a>(&self, _buffer: &'a mut [u8]) -> &'a [u8] {
        // KICC common ACK spec
        &common::RAW_DATA_ACK
    }

    fn response_nack<'a>(&self, _buffer: &'a mut [u8]) -> &'a [u8] {
        // KICC common NACK spec
        &common::RAW_DATA_NACK
    }

    fn response_device_info<'a>(
        &self,
        buffer: &'a mut [u8],
        _model_version: &'a [u8; FW_VER_LEN],
        _serial_number: &'a [u8; DEV_SN_LEN],
    ) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }

    fn alert_coin_paper_acceptor_income<'a>(
        &self,
        buffer: &'a mut [u8],
        _income: RawU24IncomeArcade,
    ) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }

    fn push_sale_slot_info<'a>(
        &self,
        buffer: &'a mut [u8],
        _port_backup: &'a CardReaderPortBackup,
    ) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }

    fn push_sale_slot_info_partial_inhibit<'a>(
        &self,
        buffer: &'a mut [u8],
        _port_backup: &'a CardReaderPortBackup,
    ) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }

    fn request_sale_slot_info<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }

    fn request_terminal_info<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }

    fn display_rom<'a>(
        &self,
        buffer: &'a mut [u8],
        _git_hash: &'a [u8; GIT_HASH_LEN],
        _terminal_id: &[u8; TID_LEN],
        _p1_card: u32,
        _p2_card: u32,
        _p1_coin: u32,
        _p2_coin: u32,
    ) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }

    fn display_hw_info<'a>(
        &self,
        buffer: &'a mut [u8],
        _model_version: &'a [u8; FW_VER_LEN],
        _serial_number: &'a [u8; DEV_SN_LEN],
        _terminal_id: &[u8; TID_LEN],
        _hw_boot_cnt: u32,
        _uptime_minutes: u32,
    ) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }

    fn display_warning<'a>(
        &self,
        buffer: &'a mut [u8],
        _warn_kind: CardTerminalDisplayWarning,
    ) -> &'a [u8] {
        // implement me for actual usage
        &buffer[0..0]
    }
}
