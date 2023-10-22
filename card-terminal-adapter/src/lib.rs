/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! This adapter is designed to separate the NDA code section from the actual packet parser/generator.
//! It also offers the possibility of connecting with different card terminal implementations in the future.
//! However, it's important to note that the initial adapter is fitted for the KICC ED785.
//! Thus other card terminal manufacturers may need to customize
//! specific firmware while considering the existing adapter as a reference.

#![no_std]
#![feature(const_trait_impl)]

pub mod types;

use types::*;

pub enum TerminalVersion {
    ArcadeSpecificLatest,
    ArcadeSpecificLegacy,
    GenericPriceIncomeType,
    Experimental,
    Unknown,
}

#[derive(Debug, defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CardTerminalError {
    /// Given checksum has fault checksum with known packet frame spec
    BadChecksum,
    /// Bad length,
    BadLength,
    /// Unsupported parameter is included on data section (text style)
    UnsupportedParameter,
    /// Basic packet frame is correct, but cannot determine internal spec
    UnsupportedSpec,
    /// Invalid Frame, it mean totally crashed data
    InvalidFrame,
    /// There is no request for the spec or not implemented
    VarientNotSupportRequest,
    /// Wrong src, suggest to check RX/TX are shorted.
    WrongSource,
    /// Failed Response
    FailedResponse,
}

#[derive(PartialEq, Eq, Clone, defmt::Format)]
pub enum CardTerminalRxCmd {
    /// Ack signal
    Ack,
    /// Nack signal
    Nack,
    RequestDeviceInfo,
    /// For generic credit card terminal (not arcade specific customized version)
    AlertPaymentIncomePrice(RawU24Price),
    /// For arcade speicific customied version credit card terminal
    AlertPaymentIncomeArcade(RawU24IncomeArcade),
    /// Detail pakcet data should be parsed with additional function call.
    /// using additional function call for avoid queue size being huge.
    ResponseSaleSlotInfo,
    /// 0xFB 0x14 0x02
    /// Detail pakcet data should be parsed with additional function call.
    /// using additional function call for avoid queue size being huge.
    ResponseTerminalInfo,
}

#[derive(PartialEq, Eq, Clone, defmt::Format)]
pub enum CardTerminalTxCmd {
    /// Ack signal
    Ack,
    /// Nack signal
    Nack,
    /// Generate info from static/const data (firmware version and device name)
    ResponseDeviceInfo,
    /// For arcade speicific customied version credit card terminal
    PushCoinPaperAcceptorIncome(RawU24IncomeArcade),
    /// Get sale slot from card terminal
    RequestSaleSlotInfo,
    /// Overwrite sale slot info to card terminal
    /// It's for rollback or treat as inihibit action
    PushSaleSlotInfo,
    /// Mixed request of PushSaleSlotInfo,
    /// todo! - This queue element should be touched later
    PushSaleSlotInfoPartialInhibit(RawPlayersInhibit),
    /// Request terminal info, include TID, terminal program version etc.
    RequestTerminalInfo,
    /// Display ROM (P1/P2 Card and Coin Meter)
    DisplayRom,
    /// Display HW Info (S/N, FW version ... etc)
    DisplayHwInfo,
    /// Display Warnings
    DisplayWarning(CardTerminalDisplayWarning),
}

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format)]
pub enum CardTerminalDisplayWarning {
    RequireArcadeSpecificVersion,
    RequireLatestTerminalVersion,
    WarnExperimentalVesion,
    WarnUnknown,
}

pub const TID_LEN: usize = 10;
pub const FW_VER_LEN: usize = 5;
pub const DEV_SN_LEN: usize = 12;
pub const GIT_HASH_LEN: usize = 9;

#[const_trait]
pub trait CardTerminalConst {
    fn is_nda() -> bool;
}

pub trait CardTerminalRxParse {
    fn pre_parse_common(&self, raw: &[u8]) -> Result<CardTerminalRxCmd, CardTerminalError>;

    // Parse ResponseSaleSlotInfo with after pre_parse_common call
    fn post_parse_response_sale_slot_info(
        &self,
        raw: &[u8],
    ) -> Result<CardReaderPortBackup, CardTerminalError>;

    // Parse ResponseTerminalInfo with after pre_parse_common call
    fn post_parse_response_terminal_info(
        &self,
        raw: &[u8],
    ) -> Result<(TerminalVersion, RawTerminalId), CardTerminalError>;
}

pub trait CardTerminalTxGen {
    // Generate ACK signal to send
    fn response_ack<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    // Generate NACK signal to send
    fn response_nack<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    /// Generate ResponseDeviceInfo signal to send
    /// Response for requesting device information (RequestDeviceInfo)
    fn response_device_info<'a, 'b>(
        &self,
        buffer: &'a mut [u8],
        model_version: &'b [u8; FW_VER_LEN],
        serial_number: &'b [u8; DEV_SN_LEN],
    ) -> &'a [u8];

    /// Generate PushCoinPaperAcceptorIncome signal to send
    fn alert_coin_paper_acceptor_income<'a>(
        &self,
        buffer: &'a mut [u8],
        income: RawU24IncomeArcade,
    ) -> &'a [u8];

    /// Generate PushSaleSlotInfo signal to send
    /// This action send for all slots without modification
    fn push_sale_slot_info<'a>(
        &self,
        buffer: &'a mut [u8],
        port_backup: &'a CardReaderPortBackup,
    ) -> &'a [u8];

    /// Generate PushSaleSlotInfoPartialInhibit signal to send
    /// This action send with modificated slots to inhibit sale slot for inhibit behavior
    fn push_sale_slot_info_partial_inhibit<'a>(
        &self,
        buffer: &'a mut [u8],
        port_backup: &'a CardReaderPortBackup,
    ) -> &'a [u8];

    /// Generate RequestSaleSlotInfo signal to send
    fn request_sale_slot_info<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    /// Generate RequestTerminalInfo signal to send
    fn request_terminal_info<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    /// Generate DisplayRom signal to send
    /// Display card / coin count for player 1 and 2 on LCD of card terminal.
    fn display_rom<'a>(
        &self,
        buffer: &'a mut [u8],
        git_hash: &'a [u8; GIT_HASH_LEN],
        terminal_id: &[u8; TID_LEN],
        p1_card: u32,
        p2_card: u32,
        p1_coin: u32,
        p2_coin: u32,
    ) -> &'a [u8];

    /// Generate DisplayHwInfo signal to send
    /// Display hardware information, boot count, uptime and etc.
    fn display_hw_info<'a, 'b>(
        &self,
        buffer: &'a mut [u8],
        model_version: &'b [u8; FW_VER_LEN],
        serial_number: &'b [u8; DEV_SN_LEN],
        terminal_id: &[u8; TID_LEN],
        hw_boot_cnt: u32,
        uptime_minutes: u32,
    ) -> &'a [u8];

    /// Generate DisplayWarning signal to send
    /// Display warning that need to update to latest terminal version firmware or something
    fn display_warning<'a>(
        &self,
        buffer: &'a mut [u8],
        warn_kind: CardTerminalDisplayWarning,
    ) -> &'a [u8];
}
