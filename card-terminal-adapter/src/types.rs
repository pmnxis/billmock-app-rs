/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! # NDA feature + EEPROM (novella) feature related types.
use static_assertions::*;
use zeroable::Zeroable;

pub struct RawU24Price(pub [u8; 3]);

impl From<u32> for RawU24Price {
    fn from(value: u32) -> Self {
        // big endian
        let value = value.min((1 << 24) - 1);

        Self([
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        ])
    }
}

impl From<RawU24Price> for u32 {
    fn from(value: RawU24Price) -> Self {
        // big endian
        ((value.0[0] as u32) << 16) | ((value.0[1] as u32) << 8) | (value.0[0] as u32)
    }
}

#[derive(Clone, Zeroable, PartialEq, Eq, Debug)]
pub struct IncomeArcadeRequest {
    pub port: u8,
    pub pulse_count: u16,
    pub pulse_duration: u16,
}

/// [port: 4b, pulse_count: msb-4b], [pulse_count: 6b-lsb, pulse_duration: msb-2]
/// [pulse_duration: 8b-lsb]
#[derive(Clone, Zeroable)]
pub struct RawU24IncomeArcade([u8; 3]);

impl From<IncomeArcadeRequest> for RawU24IncomeArcade {
    fn from(value: IncomeArcadeRequest) -> Self {
        let pulse_count = value.pulse_count.min(999);
        let pulse_duration = value.pulse_duration.min(999);

        Self([
            (value.port << 4) | ((pulse_count >> 6) as u8 & 0xF),
            ((pulse_count as u8) << 2) | ((pulse_duration >> 8) as u8 & 0x3),
            pulse_duration as u8,
        ])
    }
}

impl From<RawU24IncomeArcade> for IncomeArcadeRequest {
    fn from(value: RawU24IncomeArcade) -> Self {
        Self {
            port: value.0[0] >> 4,
            pulse_count: (((value.0[0] & 0x0F) as u16) << 6) | (value.0[1] >> 2) as u16,
            pulse_duration: (((value.0[1] & 0x03) as u16) << 8) | value.0[2] as u16,
        }
    }
}

pub struct RawPlayersInhibit {
    pub p1: u8,
    pub p2: u8,
}

#[repr(C)]
#[derive(Clone, Zeroable)]
pub struct RawTerminalId {
    pub normal: [u8; 10],
    pub extend: [u8; 3],
}
assert_eq_size!(RawTerminalId, [u8; 13]);

#[derive(Clone, Zeroable)]
pub struct RawPortPulseCountDuration {
    pub inner: u32,
}

// #[derive(Clone)]
// pub struct RawGameNumPrice {
//     pub
// }

pub struct SlotPriceGameNum {
    pub price: u32,
    pub game_num: u16,
}

#[derive(Clone, Zeroable)]
pub struct RawU32SlotPriceGameNum(u32);

impl From<SlotPriceGameNum> for RawU32SlotPriceGameNum {
    fn from(value: SlotPriceGameNum) -> Self {
        Self {
            0: ((value.price.max(99_999) & ((1 << 17) - 1)) << 10)
                | ((value.game_num.max(999) & ((1 << 10) - 1)) as u32),
        }
    }
}

#[derive(Clone, Zeroable)]
pub struct RawCardPortBackup {
    // is enabled?
    pub is_enabled: bool,
    // Contains pulse count, pulse duration
    pub raw_extended: RawU32SlotPriceGameNum,
    pub raw_minimum: RawU24IncomeArcade,
}
assert_eq_size!(RawCardPortBackup, [u8; 8]);

impl From<(SlotPriceGameNum, IncomeArcadeRequest)> for RawCardPortBackup {
    fn from((extended, minimum): (SlotPriceGameNum, IncomeArcadeRequest)) -> Self {
        Self {
            is_enabled: extended.game_num != 0,
            raw_extended: extended.into(),
            raw_minimum: minimum.into(),
        }
    }
}

impl RawCardPortBackup {
    pub fn empty_slot() -> Self {
        Self::zeroed()
    }
}

#[derive(Clone, Zeroable)]
pub struct CardReaderPortBackup {
    pub raw_card_port_backup: [RawCardPortBackup; 4],
}

impl CardReaderPortBackup {
    pub fn empty_slot() -> Self {
        Self::zeroed()
    }
}

assert_eq_size!(CardReaderPortBackup, [u8; 32]);
