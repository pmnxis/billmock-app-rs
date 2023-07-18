/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PriceVariant {
    PriceType0 = 0,
    PriceType1 = 1,
    PriceType2 = 2,
    PriceType3 = 3,
}

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum TimingVariant {
    PulseTiming50Millis = 0,
    PulseTiming100Millis = 1,
    PulseTiming200Millis = 2,
    PulseTimingAuto = 3,
}

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AppMode {
    BypassMode = 0,
    DualEmulationMode = 1,
    UnknownMode2 = 2,
    UnknownMode3 = 3,
    /* TBD */
}
