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

pub enum CardTerminalTxCmd {
    /// Ack signal
    Ack,
    /// Nack signal
    Nack,
    /// Generate info from static/const data (firmware version and device name)
    ResponseDeviceInfo,
    /// For arcade speicific customied version credit card terminal
    PushCoinPaperAcceptorIncome(RawU24IncomeArcade),
    RequestSaleSlotInfo,
    PushSaleSlotInfo,
    /// Mixed request of PushSaleSlotInfo,
    PushSaleSlotInfoPartialInhibit(RawPlayersInhibit),

    RequestTerminalInfo,
    /// Display
    DisplayRom,

    DisplayHwInfo,

    DisplayWarning(CardTerminalDisplayWarning),
}

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

pub trait CardTerminalRxParse {
    fn pre_parse_common(&self, raw: &[u8]) -> Result<CardTerminalRxCmd, CardTerminalError>;

    fn post_parse_response_sale_slot_info(
        &self,
        raw: &[u8],
    ) -> Result<CardReaderPortBackup, CardTerminalError>;

    fn post_parse_response_terminal_info(
        &self,
        raw: &[u8],
    ) -> Result<(TerminalVersion, RawTerminalId), CardTerminalError>;
}

pub trait CardTerminalTxGen {
    fn response_ack<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    fn response_nack<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    // OK
    /// Response for requesting device information
    fn response_device_info<'a>(
        &self,
        buffer: &'a mut [u8],
        model_version: &'a [u8; FW_VER_LEN],
        serial_number: &'a [u8; DEV_SN_LEN],
    ) -> &'a [u8];

    fn alert_coin_paper_acceptor_income<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    // OK
    fn request_sale_slot_info<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    // OK
    fn request_terminal_info<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8];

    // OK
    /// Display card / coin count for player 1 and 2 on LCD of card terminal.
    fn display_rom<'a>(
        &self,
        buffer: &'a mut [u8],
        git_hash: &'a [u8; GIT_HASH_LEN],
        terminal_id: &'a [u8; TID_LEN],
        p1_card: u32,
        p2_card: u32,
        p1_coin: u32,
        p2_coin: u32,
    ) -> &'a [u8];

    // OK
    /// Display hardware information, boot count, uptime and etc.
    fn display_hw_info<'a>(&self, buffer: &'a mut [u8], hw_boot_cnt: u32, minutes: u32)
        -> &'a [u8];

    // OK
    /// Display warning that need to update to latest terminal version firmware or something
    fn display_warning<'a>(
        &self,
        buffer: &'a mut [u8],
        warn_kind: CardTerminalDisplayWarning,
    ) -> &'a [u8];
}
