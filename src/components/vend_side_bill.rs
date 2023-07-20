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

pub struct VendSideBill {
    out_inhibit: BufferedOpenDrain,
    in_vend: BufferedWait,
    in_jam: BufferedWait,
}

impl VendSideBill {
    pub const fn new(
        out_inhibit: Output<'static, AnyPin>,
        in_vend: ExtiInput<'static, AnyPin>,
        in_vend_event: InputPortKind,
        in_jam: ExtiInput<'static, AnyPin>,
        in_jam_event: InputPortKind,
        mpsc_ch: &'static InputEventChannel,
        timing: &'static DualPoleToggleTiming,
    ) -> VendSideBill {
        Self {
            out_inhibit: BufferedOpenDrain::new(out_inhibit, timing),
            in_vend: BufferedWait::new(in_vend, in_vend_event, mpsc_ch),
            in_jam: BufferedWait::new(in_jam, in_jam_event, mpsc_ch),
        }
    }
}
