/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{AnyPin, Output};

use crate::semi_layer::buffered_opendrain::BufferedOpenDrain;
use crate::semi_layer::buffered_wait::{BufferedWait, InputEventChannel, InputPortKind};
use crate::semi_layer::timing::DualPoleToggleTiming;

pub struct StartButton {
    in_switch: BufferedWait,
    out_led: BufferedOpenDrain,
}

impl StartButton {
    pub const fn new(
        in_switch: ExtiInput<'static, AnyPin>,
        in_switch_event: InputPortKind,
        out_led: Output<'static, AnyPin>,
        mpsc_ch: &'static InputEventChannel,
        timing: &'static DualPoleToggleTiming,
    ) -> Self {
        Self {
            in_switch: BufferedWait::new(in_switch, in_switch_event, mpsc_ch),
            out_led: BufferedOpenDrain::new(out_led, timing),
        }
    }
}
