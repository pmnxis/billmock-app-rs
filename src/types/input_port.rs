/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use num_enum::TryFromPrimitive;

use crate::semi_layer::buffered_wait::RawInputPortKind;
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

impl From<InputPortKind> for RawInputPortKind {
    fn from(value: InputPortKind) -> Self {
        // infallable
        // Self::try_from().unwrap()
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
