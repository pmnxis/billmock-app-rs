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
use crate::semi_layer::buffered_wait::buffered_wait_spawn;
use crate::semi_layer::buffered_wait::{BufferedWait, InputEventChannel, RawInputPortKind};
use crate::semi_layer::timing::SharedToggleTiming;
use crate::types::buffered_opendrain_kind::BufferedOpenDrainKind;
use crate::types::input_port::InputPortKind;
use crate::types::player::Player;

pub struct VendSideBill {
    pub out_inhibit: BufferedOpenDrain,
    in_vend: BufferedWait,
    in_start_jam: BufferedWait,
}

impl VendSideBill {
    pub const fn new(
        player: Player,
        out_inhibit: Output<'static, AnyPin>,
        in_vend: ExtiInput<'static, AnyPin>,
        in_start_jam: ExtiInput<'static, AnyPin>,
        mpsc_ch: &'static InputEventChannel,
        shared_timing: &'static SharedToggleTiming,
    ) -> VendSideBill {
        let inhibit_str: &'static str = BufferedOpenDrainKind::VendSideInhibit(player).const_str();
        let (vend_p, vend_str): (RawInputPortKind, &'static str) =
            InputPortKind::Vend1P.to_raw_and_const_str(player);
        let (snj_p, snj_str): (RawInputPortKind, &'static str) =
            InputPortKind::StartJam1P.to_raw_and_const_str(player);

        Self {
            out_inhibit: BufferedOpenDrain::new(out_inhibit, shared_timing, inhibit_str),
            in_vend: BufferedWait::new(in_vend, mpsc_ch, vend_p, vend_str),
            in_start_jam: BufferedWait::new(in_start_jam, mpsc_ch, snj_p, snj_str),
        }
    }

    pub fn start_tasks(&'static self, spawner: &Spawner) {
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.out_inhibit)));
        unwrap!(spawner.spawn(buffered_wait_spawn(&self.in_vend)));
        unwrap!(spawner.spawn(buffered_wait_spawn(&self.in_start_jam)));
    }
}
