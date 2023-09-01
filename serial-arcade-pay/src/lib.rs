/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! # Common generic serial type payment method interface
//!
//! NDA device and example devices softwares should have same interface on same app code.
//! This interface has limited feature or fields for commonrize and hiding NDA protocol.

#![no_std]

#[derive(Debug, defmt::Format, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct GenericIncomeInfo {
    pub player: Option<u8>,
    pub price: Option<u32>,
    pub signal_count: Option<u16>,
    pub pulse_duration: Option<u16>,
}

impl Default for GenericIncomeInfo {
    fn default() -> Self {
        Self {
            player: None,
            price: None,
            signal_count: None,
            pulse_duration: None,
        }
    }
}

#[derive(Debug, defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TinyGenericInhibitInfo(u8);

impl TinyGenericInhibitInfo {
    pub fn set_flag(&mut self, player_number: usize, flag: bool) {
        if 1 <= player_number && player_number <= 4 {
            match flag {
                true => {
                    self.0 |= 0x1 << (player_number - 1);
                }
                false => {
                    self.0 &= !(0x1 << (player_number - 1));
                }
            }
        }
    }

    pub fn get_raw(&self) -> u8 {
        self.0
    }

    pub fn is_inhibit(&self, player_number: usize) -> bool {
        if 1 <= player_number && player_number <= 4 {
            self.0 & (0x1 << (player_number - 1)) != 0
        } else {
            false
        }
    }

    pub fn as_tuple(&self) -> (bool, bool, bool, bool) {
        (
            self.0 & 0x1 != 0,
            self.0 & (0x1 << 1) != 0,
            self.0 & (0x1 << 2) != 0,
            self.0 & (0x1 << 3) != 0,
        )
    }

    pub const fn const_new(p1: bool, p2: bool, p3: bool, p4: bool) -> Self {
        Self {
            0: ((p4 as u8) << 3) | ((p3 as u8) << 2) | ((p2 as u8) << 1) | (p1 as u8),
        }
    }

    pub fn new(p1: bool, p2: bool, p3: bool, p4: bool) -> Self {
        Self::const_new(p1, p2, p3, p4)
    }

    pub fn new_from_u8(value: u8) -> Self {
        Self(value)
    }
}

impl Default for TinyGenericInhibitInfo {
    fn default() -> Self {
        Self { 0: 0 }
    }
}

#[derive(Debug, defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum GenericPaymentRecv {
    /// The device alert alive itself
    Heartbeat,
    /// Ack signal
    Ack,
    /// Nack signal
    Nack,
    /// Payment income
    Income(GenericIncomeInfo),
    /// Busy by user behavior or something else
    SetBusyState(bool),
    /// Check Inhibit state
    CheckInhibit(TinyGenericInhibitInfo),
    /// Failed payment
    Failed,
    /// Unknown
    Unknown,
}

#[derive(defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum GenericPaymentRequest {
    /// Heratbeat signal from MCU to card reader device
    Heartbeat,
    /// Common ACK
    Ack,
    /// Common NACK
    Nack,
    /// Request current busy state manually
    CheckBusy,
    /// Request current inhibit state manually
    CheckInhibit,
    /// Request deny any card payment globally
    SetGlobalInhibit(bool),
    /// Request
    SetInhibit(TinyGenericInhibitInfo),
}

#[derive(Debug, defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SerialArcadeError {
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

/// Common generic serial type payment method interface
pub trait SerialArcadePay: Sized + Clone + PartialEq + defmt::Format {
    /// parse rx buf and return degraded data and varient info(Self)
    fn parse_rx(
        raw_data: &[u8],
        raw_len: usize,
    ) -> Result<(GenericPaymentRecv, Self), SerialArcadeError>;

    /// generate tx buf data by request and varient info(Self)
    fn generate_tx(
        &self,
        request: &GenericPaymentRequest,
        tx_buffer: &mut [u8],
    ) -> Result<usize, SerialArcadeError>;

    /// tell it's NDA or not, this feature is required for depenency injection
    fn is_nda() -> bool;
}
