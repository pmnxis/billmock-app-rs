/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use num_enum::{IntoPrimitive, TryFromPrimitive};

// Dip Switch spec
// DipSwitch was assigned as Price/Timing/Mode0v2 in hardware 0.2 spec.
// But in DipSwitch is assigned as Inhibit/Timing/Mode0v3 in hardware 0.3 spec.

/// ## Price dip switch configuration used in HW spec 0.2 [DEPRECATED]
///
/// | PRICE0 (`1`)  | PRICE1 (`2`)  | Configuration                 |
/// | :-----------: | :-----------: | ----------------------------- |
/// | `0`           |  `0`          | Auto                          |
/// | `0`           |  `1`          | Reserved                      |
/// | `1`           |  `0`          | Force 1 signal per 500 KRW    |
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
/// | Inhibit0 (`1`)| Inhibit1 (`2`)| Configuration                   |
/// | :-----------: | :-----------: | ------------------------------- |
/// | `0`           |  `0`          | Normal (No force overriding)    |
/// | `0`           |  `1`          | Override inhibit on 1P as force |
/// | `1`           |  `0`          | Override inhibit on 2P as force |
/// | `1`           |  `1`          | Override inhibit globally       |
///
/// - By override for each player by dip switch setting, regardless of game I/O signal,
///  it is forcibly set to inhibit state.
/// - Even if there is no override in the dip switch settings,
///  if inhibit is enabled on the host game I/O side, inhibit for each player is activated.
/// - This Inhibit DIP Switch setting can be used to prohibit currency acquisition
///  of a device that has under the maintenance in the field engineer.
#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
#[allow(dead_code)]
pub enum InhibitOverride {
    /// `00` : default value
    Normal,
    /// `01` : Override 1P inhibit
    ForceInhibit1P,
    /// `10` : Override 2P inhibit
    ForceInhibit2P,
    /// `11` : Override global inhibit (1P/2P)
    ForceInhibitGlobal,
}

/// ## Timing dip switch configuration used in HW spec 0.2 and 0.3
///
/// | TIMING0 (`3`) | TIMING1 (`4`) | Configuration                 |
/// | :-----------: | :-----------: | ----------------------------- |
/// | `0`           |  `0`          | Auto                          |
/// | `0`           |  `1`          | Force 50mS active low         |
/// | `1`           |  `0`          | Force 100mS active low        |
/// | `1`           |  `1`          | Force 200mS active low        |
/// - Timing SW `00` (Auto), for the active-low output signal,
///  the pulse duration provided by serial communication or
///  the pulse duration measurement value of parallel communication (legacy coin & bill acceptor)
///  is set separately according to the signal source.
///  If both are unavailable, the default value (100 mS) will be used.
///
/// - Timing SW `01`, `10`, `11` ignores the pulse duration of all signal sources and
///  fixes it to one of 50 mS, 100 mS, and 200 mS and outputs it.
#[derive(TryFromPrimitive, IntoPrimitive)]
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

/// ## Application mode dip switch configuration used in HW spec 0.2 [DEPRECATED]
///
/// | MODE0 (`5`)   | MODE1 (`6`)   | Configuration                 |
/// | :-----------: | :-----------: | ----------------------------- |
/// | `0`           |  `0`          | Bypass mode (Default)         |
/// | `0`           |  `1`          | Dual emulation mode           |
/// | `1`           |  `0`          | Start signal decide vend output direction for payment income from serial communication |
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
/// | `0`         |  `1`        | Start Signal | Yes, Start signal decide vend output direction for payment income from serial communication |
/// | `1`         |  `0`        | Jam Signal   | No, Jam signal bypass to jam(game pcb) side output    |
/// | `1`         |  `1`        | Jam Signal   | Yes, Extra serial port is forcely bind to 2P output, default port to 1P |
///
/// - MODE0 (5) : Swap `start` and `jam` input signal on vend side, default definition is start.
/// - MODE1 (6) : Special feature disable or enable
#[derive(TryFromPrimitive, IntoPrimitive)]
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
    /// `11` : BypassJamAndExtraSerialPayment
    /// Bypass JAM (swapped logically). JAM signal bypass to host(game pcb) side output.
    /// And independent of swapping signal, extra serial port is forcely bind to 2P output, default port to 1P
    BypassJamAndExtraSerialPayment = 3,
}
