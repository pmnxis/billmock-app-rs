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

#[derive(defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum GenericPaymentRequest {
    /// Heratbeat signal from MCU to card reader device
    Heartbeat,
    /// Common ACK
    Ack,
    /// Common NACK
    Nack,
    /// Request deny any card payment
    SetInhibit,
    /// Request allow any card payment
    ClearInhibit,
}

#[derive(Debug, defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
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
    SetBusy,
    /// Busy state is cleared
    ClearBusy,
    /// Failed payment
    Failed,
    /// Unknown
    Unknown,
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
}

pub trait SerialArcadePay<SerialArcadeSpec: Sized + Clone + PartialEq + defmt::Format>:
    Sized
{
    fn parse_rx(
        raw_data: &[u8],
        raw_len: usize,
    ) -> Result<(GenericPaymentRecv, SerialArcadeSpec), SerialArcadeError>;

    fn generate_tx(
        request: GenericPaymentRequest,
        tx_buffer: &mut [u8],
    ) -> Result<usize, SerialArcadeError>;
}
