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

pub struct HostSideBill {
    in_inhibit: BufferedWait,
    out_busy: BufferedOpenDrain,
    out_vend: BufferedOpenDrain,
    out_jam: BufferedOpenDrain,
    out_start: BufferedOpenDrain,
}

impl HostSideBill {
    pub const fn new(
        in_inhibit: ExtiInput<'static, AnyPin>,
        in_inhibit_event: InputPortKind,
        out_busy: Output<'static, AnyPin>,
        out_vend: Output<'static, AnyPin>,
        out_jam: Output<'static, AnyPin>,
        out_start: Output<'static, AnyPin>,
        mpsc_ch: &'static InputEventChannel,
        timing: &'static DualPoleToggleTiming,
    ) -> Self {
        Self {
            in_inhibit: BufferedWait::new(in_inhibit, in_inhibit_event, mpsc_ch),
            out_busy: BufferedOpenDrain::new(out_busy, timing),
            out_vend: BufferedOpenDrain::new(out_vend, timing),
            out_jam: BufferedOpenDrain::new(out_jam, timing),
            out_start: BufferedOpenDrain::new(out_start, timing),
        }
    }
}
