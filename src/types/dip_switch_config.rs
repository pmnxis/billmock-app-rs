/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use bit_field::BitField;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::boards::{PLAYER_1_INDEX, PLAYER_2_INDEX, PLAYER_INDEX_MAX};
use crate::semi_layer::timing::ToggleTiming;

// Dip Switch spec
// DipSwitch was assigned as Price/Timing/Mode0v2 in hardware 0.2 spec.
// But in DipSwitch is assigned as Inhibit/Timing/Mode0v3 in hardware 0.3 spec.

/// ## Price dip switch configuration used in HW spec 0.2 [DEPRECATED]
///
/// | PRICE0 (`1`)  | PRICE1 (`2`)  | Configuration                 |
/// | :-----------: | :-----------: | ----------------------------- |
/// | `0`           |  `0`          | Auto                          |
/// | `1`           |  `0`          | Reserved                      |
/// | `0`           |  `1`          | Force 1 signal per 500 KRW    |
/// | `1`           |  `1`          | Force 1 signal per 1000 KRW   |
///
/// - `10` and `11` : Ignore signal count field comes from serial communication,
///  decide number of output signal count from price field.
#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
#[allow(dead_code)]
pub enum PriceReflection {
    /// `00` : default value
    Auto = 0,
    /// `01` : not decided
    Reserved01 = 1,
    /// Ignore signal count field comes from serial communication,
    /// `10` : Force 1 signal per 500 KRW
    Force500Krw = 2,
    /// Ignore signal count field comes from serial communication,
    /// `11` : Force 1 signal per 1000 KRW
    Force1000Krw = 3,
}

/// ## Inhibit override dip switch configuration used in HW spec 0.3
///
/// | Inhibit0 (`1`)| Inhibit1P (`2`)| Configuration                   |
/// | :-----------: | :-----------: | ------------------------------- |
/// | `0`           |  `0`          | Normal (No force overriding)    |
/// | `1`           |  `0`          | Override inhibit on 1P as force |
/// | `0`           |  `1`          | Override inhibit on 2P as force |
/// | `1`           |  `1`          | Override inhibit globally       |
///
/// - By override for each player by dip switch setting, regardless of game I/O signal,
///  it is forcibly set to inhibit state.
/// - Even if there is no override in the dip switch settings,
///  if inhibit is enabled on the host game I/O side, inhibit for each player is activated.
/// - This Inhibit DIP Switch setting can be used to prohibit currency acquisition
///  of a device that has under the maintenance in the field engineer.
#[derive(TryFromPrimitive, IntoPrimitive, PartialEq, PartialOrd, Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)]
pub enum InhibitOverride {
    /// `00` : default value
    Normal = 0b00,
    /// `01` : Override 1P inhibit
    ForceInhibit1P = 0b01,
    /// `10` : Override 2P inhibit
    ForceInhibit2P = 0b10,
    /// `11` : Override global inhibit (1P/2P)
    ForceInhibitGlobal = 0b11,
}

impl InhibitOverride {
    pub const fn default() -> Self {
        Self::Normal
    }

    pub fn check_new_inhibited(&self, previous: &Self) -> (bool, bool) {
        let self_u8: u8 = *self as u8;
        let previous_u8 = *previous as u8;
        let masked = (self_u8 ^ previous_u8) & self_u8;

        (
            masked.get_bit(PLAYER_1_INDEX),
            masked.get_bit(PLAYER_2_INDEX),
        )
    }

    pub fn check_new_released(&self, previous: &Self) -> (bool, bool) {
        let self_u8 = *self as u8;
        let previous_u8 = *previous as u8;
        let masked = (self_u8 ^ previous_u8) & ((!self_u8) & ((1 << PLAYER_INDEX_MAX) - 1));

        (
            masked.get_bit(PLAYER_1_INDEX),
            masked.get_bit(PLAYER_2_INDEX),
        )
    }

    pub fn check_changed(&self, previous: &Self) -> (Option<bool>, Option<bool>) {
        let self_u8 = *self as u8;
        let previous_u8 = *previous as u8;
        let masked = self_u8 ^ previous_u8;

        (
            masked
                .get_bit(PLAYER_1_INDEX)
                .then_some(self_u8.get_bit(PLAYER_1_INDEX)),
            masked
                .get_bit(PLAYER_2_INDEX)
                .then_some(self_u8.get_bit(PLAYER_2_INDEX)),
        )
    }
}

impl defmt::Format for InhibitOverride {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            InhibitOverride::Normal => defmt::write!(fmt, "Normal"),
            InhibitOverride::ForceInhibit1P => defmt::write!(fmt, "ForceInhibit1P"),
            InhibitOverride::ForceInhibit2P => defmt::write!(fmt, "ForceInhibit2P"),
            InhibitOverride::ForceInhibitGlobal => defmt::write!(fmt, "ForceInhibitGlobal"),
        }
    }
}

/// ## Timing dip switch configuration used in HW spec 0.2 and 0.3
///
/// | TIMING0 (`3`) | TIMING1 (`4`) | Configuration                 |
/// | :-----------: | :-----------: | ----------------------------- |
/// | `0`           |  `0`          | Auto                          |
/// | `1`           |  `0`          | Force 50mS active low         |
/// | `0`           |  `1`          | Force 100mS active low        |
/// | `1`           |  `1`          | Force 200mS active low        |
///
/// - Timing SW `00` (Auto), for the active-low output signal,
///  the pulse duration provided by serial communication or
///  the pulse duration measurement value of parallel communication (legacy coin & bill acceptor)
///  is set separately according to the signal source.
///  If both are unavailable, the default value (100 mS) will be used.
///
/// - Timing SW `01`, `10`, `11` ignores the pulse duration of all signal sources and
///  fixes it to one of 50 mS, 100 mS, and 200 mS and outputs it.
#[derive(TryFromPrimitive, IntoPrimitive, PartialEq, PartialOrd)]
#[repr(u8)]
#[allow(dead_code)]
pub enum TimingOverride {
    /// Timing SW `00` (Auto), for the active-low output signal,
    /// the pulse duration provided by serial communication or
    /// the pulse duration measurement value of parallel communication (legacy coin & bill acceptor)
    /// is set separately according to the signal source.
    /// If both are unavailable, the default value (100 mS) will be used.
    PulseTimingAuto = 0,
    /// Ignores the pulse duration of all signal sources and fixes it to 50 milli seconds.
    PulseTiming50Millis = 1,
    /// Ignores the pulse duration of all signal sources and fixes it to 100 milli seconds.
    PulseTiming100Millis = 2,
    /// Ignores the pulse duration of all signal sources and fixes it to 200 milli seconds.
    PulseTiming200Millis = 3,
}

impl TimingOverride {
    pub const fn default() -> Self {
        Self::PulseTimingAuto
    }

    pub const fn get_toggle_timing(&self) -> ToggleTiming {
        match self {
            Self::PulseTimingAuto => ToggleTiming::default(),
            Self::PulseTiming50Millis => ToggleTiming {
                high_ms: 50,
                low_ms: 50,
            },
            Self::PulseTiming100Millis => ToggleTiming {
                high_ms: 100,
                low_ms: 100,
            },
            Self::PulseTiming200Millis => ToggleTiming {
                high_ms: 200,
                low_ms: 200,
            },
        }
    }

    pub const fn is_override_force(&self) -> bool {
        !matches!(self, Self::PulseTimingAuto)
    }
}

impl defmt::Format for TimingOverride {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            TimingOverride::PulseTimingAuto => defmt::write!(fmt, "PulseTimingAuto"),
            TimingOverride::PulseTiming50Millis => defmt::write!(fmt, "PulseTiming50Millis"),
            TimingOverride::PulseTiming100Millis => defmt::write!(fmt, "PulseTiming100Millis"),
            TimingOverride::PulseTiming200Millis => defmt::write!(fmt, "PulseTiming200Millis"),
        }
    }
}

/// ## Application mode dip switch configuration used in HW spec 0.2 [DEPRECATED]
///
/// | MODE0 (`5`)   | MODE1 (`6`)   | Configuration                 |
/// | :-----------: | :-----------: | ----------------------------- |
/// | `0`           |  `0`          | Bypass mode (Default)         |
/// | `1`           |  `0`          | Dual emulation mode           |
/// | `0`           |  `1`          | Start signal decide vend output direction for payment income from serial communication |
/// | `1`           |  `1`          | Reserved                      |
///
/// Since this setting is no longer used, a detailed description is omitted.
#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
#[allow(dead_code)]
pub enum AppMode {
    BypassMode = 0,
    DualEmulationMode = 1,
    SeparatedStartSelect = 2,
    UnknownMode3 = 3,
}

/// ## Application mode dip switch configuration used in HW spec 0.3
///
/// | MODE0 (`5`) | MODE1 (`6`) | Swap status  | Special Feature                                       |
/// | :---------: | :---------: | ------------ | ----------------------------------------------------- |
/// | `0`         |  `0`        | Start Signal | No, Start signal bypass to host(game pcb) side output |
/// | `1`         |  `0`        | Start Signal | Yes, Start signal decide vend output direction for payment income from serial communication |
/// | `0`         |  `1`        | Jam Signal   | No, Jam signal bypass to jam(game pcb) side output    |
/// | `1`         |  `1`        | Jam Signal   | Reserved                                              |
///
/// - MODE0 (5) : Special feature disable or enable
/// - MODE1 (6) : Swap `start` and `jam` input signal on vend side, default definition is start.
#[derive(TryFromPrimitive, IntoPrimitive, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
pub enum AppMode0V3 {
    /// `00` : BypassStart
    /// Normal mode with bypass start (default value). Start signal bypass to host(game pcb) side output.
    BypassStart = 0,
    /// `01` : StartButtonDecideSerialToVend
    /// Special mode with start button mocked.
    /// Start signal decide vend output direction for payment income from serial communication.
    StartButtonDecideSerialToVend = 1,
    /// `10` : BypassJam
    /// Normal mode with bypass JAM (swapped logically). JAM signal bypass to host(game pcb) side output.
    BypassJam = 2,
    /// `11` : BypassJamButReserved
    /// Bypass JAM (swapped logically). JAM signal bypass to host(game pcb) side output.
    /// This configuration is reserved for future usage.
    DisplayRom = 3,
}

impl AppMode0V3 {
    pub const fn default() -> Self {
        Self::BypassStart
    }
}

impl defmt::Format for AppMode0V3 {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            AppMode0V3::BypassStart => defmt::write!(fmt, "BypassStart"),
            AppMode0V3::StartButtonDecideSerialToVend => {
                defmt::write!(fmt, "StartButtonDecideSerialToVend")
            }
            AppMode0V3::BypassJam => defmt::write!(fmt, "BypassJam"),
            AppMode0V3::DisplayRom => {
                defmt::write!(fmt, "DisplayRom")
            }
        }
    }
}
