/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use num_enum::TryFromPrimitive;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, defmt::Format, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
pub enum Player {
    Undefined = 0,
    Player1 = 1,
    Player2 = 2,
}

impl Player {
    pub const fn default() -> Self {
        Self::Undefined
    }
}
