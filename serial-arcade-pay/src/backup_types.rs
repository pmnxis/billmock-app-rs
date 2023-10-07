/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! # NDA feature + EEPROM (novella) feature related types.

use static_assertions::*;

#[repr(C)]
#[derive(Clone)]
pub struct RawTerminalId {
    pub normal: [u8; 10],
    pub extend: [u8; 3],
}
assert_eq_size!(RawTerminalId, [u8; 13]);

#[derive(Clone)]
pub struct RawPortPulsePulseDuration {
    pub inner: u32,
}

#[derive(Clone)]
pub struct RawGameNumPrice {
    pub inner: u32,
}

#[derive(Clone)]
pub struct RawCardPortBackup {
    pub port_pulse_pulse_duration: RawPortPulsePulseDuration,
    pub game_num_price: RawGameNumPrice,
}
assert_eq_size!(RawCardPortBackup, [u8; 8]);

#[derive(Clone)]
pub struct CardReaderPortBackup {
    pub raw_card_port_backup: [RawCardPortBackup; 4],
}
assert_eq_size!(CardReaderPortBackup, [u8; 32]);
