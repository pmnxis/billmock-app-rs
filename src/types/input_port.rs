/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use num_enum::TryFromPrimitive;

use crate::semi_layer::buffered_wait::{InputEventKind, RawInputEvent, RawInputPortKind};
use crate::types::const_convert::*;
use crate::types::player::Player;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
pub enum InputPortKind {
    Start1P = 0,
    Start2P = 1,
    Vend1P = 2,
    Vend2P = 3,
    Jam1P = 4,
    Jam2P = 5,
    StartJam1P = 6,
    StartJam2P = 7,
    Inhibit1P = 8,
    Inhibit2P = 9,
    SvcButton = 10,
    // Ignored signal by filter function
    Nothing = 11,
}

#[cfg(debug_assertions)]
const INPUT_PORT_KIND_STRS: [&str; 12] = [
    "VendIn_1P-Start  ",
    "VendIn_2P-Start  ",
    "VendIn_1P-Vend   ",
    "VendIn_2P-Vend   ",
    "VendIn_1P-Jam    ",
    "VendIn_2P-Jam    ",
    "VendIn_1P-STR/JAM",
    "VendIn_2P-STR/JAM",
    "HostIn_1P-Inhibit",
    "HostIn_2P-Inhibit",
    "SVC_Button       ",
    "Nothing",
];

#[cfg(not(debug_assertions))]
#[rustfmt::skip]
const INPUT_PORT_KIND_STRS: [&str; 12] = [
    "P1V-iSTR",
    "P2V-iSTR",
    "P1V-iVND",
    "P2V-iVND",
    "P1V-iJAM",
    "P2V-iJAM",
    "P1V-iS/J",
    "P2V-iS/J",
    "P1H-iINH",
    "P2H-iINH",
    "iSVC_BT ",
    "iNothing",
];

// assert_eq!(InputPortKind::count(), INPUT_PORT_KIND_STRS.len());

impl defmt::Format for InputPortKind {
    fn format(&self, fmt: defmt::Formatter) {
        let idx = (*self as u8) as usize;
        if idx < INPUT_PORT_KIND_STRS.len() {
            defmt::write!(fmt, "InputPortKind::{}", INPUT_PORT_KIND_STRS[idx])
        } else {
            defmt::write!(fmt, "Unknown InputPortKind 0x{:02X}", idx as u8)
        }
    }
}

impl From<InputPortKind> for RawInputPortKind {
    fn from(value: InputPortKind) -> Self {
        value as u8
    }
}

impl const ConstFrom<InputPortKind> for RawInputPortKind {
    fn const_from(value: InputPortKind) -> Self {
        value as u8
    }
}

impl const ConstInto<RawInputPortKind> for InputPortKind {
    fn const_into(self) -> RawInputPortKind {
        RawInputPortKind::const_from(self)
    }
}

impl InputPortKind {
    pub const fn to_raw_and_const_str(self, player: Player) -> (RawInputPortKind, &'static str) {
        let idx = self as u8;

        // PartialEq doesn't support const boundary
        // if Player::Undefined == player {
        if (Player::Undefined as u8 == player as u8) || (idx == Self::SvcButton as u8) {
            let ret: RawInputPortKind = self as u8;
            (ret, INPUT_PORT_KIND_STRS[ret as usize])
        } else if Self::Nothing as u8 != idx {
            let ret = (idx + player as u8 - 1) as RawInputPortKind;
            (ret, INPUT_PORT_KIND_STRS[ret as usize])
        } else {
            (self as RawInputPortKind, self.const_str())
        }
    }

    pub const fn const_str(self) -> &'static str {
        INPUT_PORT_KIND_STRS[self as usize]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub port: InputPortKind,
    pub event: InputEventKind,
}

impl TryFrom<RawInputEvent> for InputEvent {
    type Error = num_enum::TryFromPrimitiveError<InputPortKind>;
    fn try_from(value: RawInputEvent) -> Result<Self, Self::Error> {
        let port: InputPortKind = value.port.try_into()?;
        let event: InputEventKind = value.event.into();

        Ok(Self { port, event })
    }
}

impl TryFrom<&RawInputEvent> for InputEvent {
    type Error = num_enum::TryFromPrimitiveError<InputPortKind>;
    fn try_from(value: &RawInputEvent) -> Result<Self, Self::Error> {
        let port: InputPortKind = value.port.try_into()?;
        let event: InputEventKind = value.event.into();

        Ok(Self { port, event })
    }
}
