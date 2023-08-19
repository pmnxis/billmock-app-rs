/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{AnyPin, Output};

use crate::semi_layer::buffered_opendrain::{buffered_opendrain_spawn, BufferedOpenDrain};
use crate::semi_layer::buffered_wait::{buffered_wait_spawn, BufferedWait, InputEventChannel};
use crate::semi_layer::timing::DualPoleToggleTiming;
use crate::types::const_convert::ConstInto;
use crate::types::input_port::InputPortKind;

pub struct VendSideBill {
    pub out_inhibit: BufferedOpenDrain,
    in_vend: BufferedWait,
    in_start_jam: BufferedWait,
}

impl VendSideBill {
    pub const fn new(
        out_inhibit: Output<'static, AnyPin>,
        in_vend: ExtiInput<'static, AnyPin>,
        in_vend_event: InputPortKind,
        in_start_jam: ExtiInput<'static, AnyPin>,
        in_start_jam_event: InputPortKind,
        mpsc_ch: &'static InputEventChannel,
        timing: &'static DualPoleToggleTiming,
    ) -> VendSideBill {
        Self {
            out_inhibit: BufferedOpenDrain::new(out_inhibit, timing),
            in_vend: BufferedWait::new(in_vend, in_vend_event.const_into(), mpsc_ch),
            in_start_jam: BufferedWait::new(in_start_jam, in_start_jam_event.const_into(), mpsc_ch),
        }
    }

    pub fn start_tasks(&'static self, spawner: &Spawner) {
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.out_inhibit)));
        unwrap!(spawner.spawn(buffered_wait_spawn(&self.in_vend)));
        unwrap!(spawner.spawn(buffered_wait_spawn(&self.in_start_jam)));
    }
}
