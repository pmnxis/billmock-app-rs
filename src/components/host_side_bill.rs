/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */
use core::default;

use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;

use crate::semi_layer::buffered_output::BufferedOuputPin;
use crate::semi_layer::buffered_wait::BufferedWait;

// use defmt::export::panic;
// // mutex with channel
// use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
// use embassy_sync::channel::Channel;
// use embassy_time::{Duration, Timer};
// use embedded_hal::digital::OutputPin;
// use embedded_hal_async::digital::Wait;
// use num_enum::{IntoPrimitive, TryFromPrimitive};
// use {defmt_rtt as _, panic_probe as _};

// use super::outie_signal_fsm::BufferedOuputPin;
// use super::timing::{DualPoleToggleTiming, ToggleTiming};

// pub const HOST_SIDE_INTERFACE_CH_SIZE: usize = 8;

// pub enum HostSideEvent {
//     Inhibitted,
//     Normal,
// }

// #[derive(TryFromPrimitive, IntoPrimitive)]
// #[repr(u8)]
// pub enum HostSideOutChannel {
//     Busy = 0,
//     Vend = 1,
//     Jam = 2,
//     Start = 3,
// }

pub struct HostSideBill<
    IN_INHIBIT: Wait,
    OUT_BUSY: OutputPin,
    OUT_VEND: OutputPin,
    OUT_JAM: OutputPin,
    OUT_START: OutputPin,
> {
    in_inhibit: BufferedWait<IN_INHIBIT>,
    out_busy: BufferedOuputPin<OUT_BUSY>,
    out_vend: BufferedOuputPin<OUT_VEND>,
    out_jam: BufferedOuputPin<OUT_JAM>,
    out_start: BufferedOuputPin<OUT_START>,
}

impl<
        IN_INHIBIT: Wait,
        OUT_BUSY: OutputPin,
        OUT_VEND: OutputPin,
        OUT_JAM: OutputPin,
        OUT_START: OutputPin,
    > HostSideBill<IN_INHIBIT, OUT_BUSY, OUT_VEND, OUT_JAM, OUT_START>
{
    pub fn new(
        in_inhibit: IN_INHIBIT,
        mut out_busy: OUT_BUSY,
        mut out_vend: OUT_VEND,
        mut out_jam: OUT_JAM,
        mut out_start: OUT_START,
    ) -> HostSideBill<IN_INHIBIT, OUT_BUSY, OUT_VEND, OUT_JAM, OUT_START> {
        unimplemented!()
    }
}
