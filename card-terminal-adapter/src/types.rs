/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! # NDA feature + EEPROM (novella) feature related types.
use static_assertions::*;
use zeroable::Zeroable;

#[derive(PartialEq, Eq, Clone, defmt::Format)]
pub struct RawU24Price(pub [u8; 3]);

impl From<u32> for RawU24Price {
    fn from(value: u32) -> Self {
        // big endianSlotPriceGameNum
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

#[derive(Debug, Zeroable, defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IncomeArcadeRequest {
    pub port: u8,
    pub pulse_count: u16,
    pub pulse_duration: u16,
}

/// [port: 4b, pulse_count: msb-4b], [pulse_count: 6b-lsb, pulse_duration: msb-2]
/// [pulse_duration: 8b-lsb]
#[derive(Clone, Zeroable, PartialEq, Eq)]
pub struct RawU24IncomeArcade([u8; 3]);

impl defmt::Format for RawU24IncomeArcade {
    fn format(&self, fmt: defmt::Formatter) {
        let raw_u24_income_arcade = IncomeArcadeRequest::from(self.clone());
        defmt::write!(fmt, "{:?}", raw_u24_income_arcade);
    }
}

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

impl RawU24IncomeArcade {
    pub fn get_port_num(&self) -> u8 {
        self.0[0] >> 4
    }
}

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format)]
pub struct RawPlayersInhibit {
    pub p1: bool,
    pub p2: bool,
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

pub struct SlotPriceGameNum {
    pub price: u32,
    pub game_num: u16,
}

#[derive(Clone, Zeroable)]
pub struct RawU32SlotPriceGameNum(u32);

impl From<SlotPriceGameNum> for RawU32SlotPriceGameNum {
    fn from(value: SlotPriceGameNum) -> Self {
        Self {
            0: ((value.price.min(99_999) & ((1 << 17) - 1)) << 10)
                | ((value.game_num.min(999) & ((1 << 10) - 1)) as u32),
        }
    }
}

impl From<RawU32SlotPriceGameNum> for SlotPriceGameNum {
    fn from(value: RawU32SlotPriceGameNum) -> Self {
        Self {
            price: (value.0 >> 10) & ((1 << 17) - 1),
            game_num: (value.0 & ((1 << 10) - 1)) as u16,
        }
    }
}

impl RawU32SlotPriceGameNum {
    pub fn get_game_num(&self) -> u16 {
        (self.0 & ((1 << 10) - 1)) as u16
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Zeroable, PartialEq, PartialOrd)]
pub enum SlotProperty {
    Disabled,
    Enabled,
    TemporaryDisabled,
}

#[derive(Clone, Zeroable)]
pub struct RawCardPortBackup {
    // is enabled?
    pub property: SlotProperty,
    // Contains pulse count, pulse duration
    pub raw_extended: RawU32SlotPriceGameNum,
    pub raw_minimum: RawU24IncomeArcade,
}
assert_eq_size!(RawCardPortBackup, [u8; 8]);

impl From<(SlotPriceGameNum, IncomeArcadeRequest)> for RawCardPortBackup {
    fn from((extended, minimum): (SlotPriceGameNum, IncomeArcadeRequest)) -> Self {
        Self {
            property: match extended.game_num {
                0 => SlotProperty::Disabled,
                _ => SlotProperty::Enabled,
            },
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

    // 0 is player 1, <- this is temporary decide,
    // 1 is player 2, <- this is temporary decide,
    pub fn guess_player_by_port_num(&self, port_num: u8) -> u8 {
        for backup in &self.raw_card_port_backup {
            if backup.raw_minimum.get_port_num() == port_num {
                let game_num = backup.raw_extended.get_game_num();

                return match (game_num, (game_num & 0x1) == 0x1) {
                    (0, _) => 0,
                    (_, true) => 0,
                    (_, false) => 1,
                };
            }
        }
        0
    }
}

assert_eq_size!(CardReaderPortBackup, [u8; 32]);
