/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use defmt::export::panic;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;
use heapless::Deque;
use {defmt_rtt as _, panic_probe as _};

use crate::semi_layer::buffered_opendrain::BufferedOpenDrain;
use crate::semi_layer::buffered_wait::{BufferedWait, InputEventChannel, InputPortKind};
use crate::semi_layer::timing::DualPoleToggleTiming;

pub struct StartButton<InSwitch: Wait, OutLed: OutputPin> {
    in_switch: BufferedWait<InSwitch>,
    out_led: BufferedOpenDrain<OutLed>,
}

impl<InSwitch: Wait, OutLed: OutputPin> StartButton<InSwitch, OutLed> {
    pub fn new(
        (in_switch, in_switch_event): (InSwitch, InputPortKind),
        mut out_led: OutLed,
        mpsc_ch: &'static InputEventChannel,
        timing: &'static DualPoleToggleTiming,
    ) -> StartButton<InSwitch, OutLed> {
        Self {
            in_switch: BufferedWait::new(in_switch, in_switch_event, mpsc_ch),
            out_led: BufferedOpenDrain::new(out_led, timing),
        }
    }
}
