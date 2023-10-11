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

pub mod backup_types;

use backup_types::{CardReaderPortBackup, RawTerminalId};

pub enum TerminalVersion {
    ArcadeSpecificLatest,
    ArcadeSpecificLegacy,
    GenericPriceIncomeType,
    Experimental,
}

pub struct RawU24Price(pub [u8; 3]);

pub struct RawU24IncomeArcade([u8; 3]);

pub struct RawPlayersInhibit {
    inner: u8,
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
    ResponseSaleSlotInfo(RawPlayersInhibit),
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
}

pub trait CardTerminalRxParse {
    fn pre_parse_common(&self, raw: &[u8]) -> Result<CardTerminalRxCmd, CardTerminalError>;

    fn post_parse_response_sale_slot_info(
        &self,
        packet: &[u8],
    ) -> Result<CardReaderPortBackup, CardTerminalError>;

    fn post_parse_response_terminal_info(
        &self,
        packet: &[u8],
    ) -> Result<RawTerminalId, CardTerminalError>;
}

pub trait CardTerminalTxGen {
    fn response_ack(&self);

    fn response_nack(&self);

    /// Response for requesting device information
    fn response_device_info(&self);

    fn alert_coin_paper_acceptor_income(&self);

    fn request_sale_slot_info(&self);

    fn request_terminal_info(&self);

    /// Display card / coin count for player 1 and 2 on LCD of card terminal.
    fn display_rom(&self, p1_card: u32, p2_card: u32, p1_coin: u32, p2_coin: u32, tid: &[u8]);

    /// Display hardware information, boot count, uptime and etc.
    fn display_hw_info(&self, hw_boot_cnt: u32, minutes: u32);

    /// Display warning that need to update to latest terminal version firmware or something
    fn display_warning(&self, warn_kind: CardTerminalDisplayWarning);
}
