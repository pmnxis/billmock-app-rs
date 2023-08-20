/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use num_enum::TryFromPrimitive;

use crate::semi_layer::buffered_wait::{InputEventKind, RawInputEvent, RawInputPortKind};
use crate::types::const_convert::*;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
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
}

static INPUT_PORT_KIND_STRS: [&str; 10] = [
    "Start1P",
    "Start2P",
    "Vend1P",
    "Vend2P",
    "Jam1P",
    "Jam2P",
    "StartJam1P",
    "StartJam2P",
    "Inhibit1P",
    "Inhibit2P",
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
