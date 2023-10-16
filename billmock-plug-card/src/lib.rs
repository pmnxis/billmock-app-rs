/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_std]
#![feature(const_trait_impl)]

#[cfg(test)] // for the print out internnal log in test code
extern crate std;

mod common;

use card_terminal_adapter::types::*;
use card_terminal_adapter::*;
use common::*;

// #[cfg(any(build, test))]
// mod helper;

pub struct KiccEd785Plug {}

impl CardTerminalConst for KiccEd785Plug {
    fn is_nda() -> bool {
        false
    }
}

impl CardTerminalRxParse for KiccEd785Plug {
    fn pre_parse_common(&self, raw: &[u8]) -> Result<CardTerminalRxCmd, CardTerminalError> {
        unimplemented!()
    }

    fn post_parse_response_sale_slot_info(
        &self,
        raw: &[u8],
    ) -> Result<CardReaderPortBackup, CardTerminalError> {
        unimplemented!()
    }

    fn post_parse_response_terminal_info(
        &self,
        raw: &[u8],
    ) -> Result<(TerminalVersion, RawTerminalId), CardTerminalError> {
        unimplemented!()
    }
}

impl CardTerminalTxGen for KiccEd785Plug {
    fn response_ack<'a>(&self, _buffer: &'a mut [u8]) -> &'a [u8] {
        &common::RAW_DATA_ACK
    }

    fn response_nack<'a>(&self, _buffer: &'a mut [u8]) -> &'a [u8] {
        &common::RAW_DATA_NACK
    }

    fn response_device_info<'a>(
        &self,
        buffer: &'a mut [u8],
        model_version: &'a [u8; FW_VER_LEN],
        serial_number: &'a [u8; DEV_SN_LEN],
    ) -> &'a [u8] {
        unimplemented!()
    }

    fn alert_coin_paper_acceptor_income<'a>(
        &self,
        buffer: &'a mut [u8],
        income: RawU24IncomeArcade,
    ) -> &'a [u8] {
        unimplemented!()
    }

    fn push_sale_slot_info<'a>(
        &self,
        buffer: &'a mut [u8],
        port_backup: &'a CardReaderPortBackup,
    ) -> &'a [u8] {
        unimplemented!()
    }

    fn push_sale_slot_info_partial_inhibit<'a>(
        &self,
        buffer: &'a mut [u8],
        port_backup: &'a CardReaderPortBackup,
    ) -> &'a [u8] {
        unimplemented!()
    }

    fn request_sale_slot_info<'a>(&self, _buffer: &'a mut [u8]) -> &'a [u8] {
        unimplemented!()
    }

    fn request_terminal_info<'a>(&self, _buffer: &'a mut [u8]) -> &'a [u8] {
        unimplemented!()
    }

    fn display_rom<'a>(
        &self,
        buffer: &'a mut [u8],
        git_hash: &'a [u8; GIT_HASH_LEN],
        terminal_id: &'a [u8; TID_LEN],
        p1_card: u32,
        p2_card: u32,
        p1_coin: u32,
        p2_coin: u32,
    ) -> &'a [u8] {
        unimplemented!()
    }

    fn display_hw_info<'a>(
        &self,
        buffer: &'a mut [u8],
        model_version: &'a [u8; FW_VER_LEN],
        serial_number: &'a [u8; DEV_SN_LEN],
        terminal_id: &'a [u8; TID_LEN],
        hw_boot_cnt: u32,
        uptime_minutes: u32,
    ) -> &'a [u8] {
        unimplemented!()
    }

    fn display_warning<'a>(
        &self,
        _buffer: &'a mut [u8],
        warn_kind: CardTerminalDisplayWarning,
    ) -> &'a [u8] {
        unimplemented!()
    }
}
