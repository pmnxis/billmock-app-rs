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
/// deprecated from hardware version 0.3
#[allow(dead_code)]
pub struct StartButton {
    in_switch: BufferedWait,
    out_led: BufferedOpenDrain,
}

#[allow(dead_code)]
impl StartButton {
    pub const fn new(
        player: Player,
        in_switch: ExtiInput<'static, AnyPin>,
        out_led: Output<'static, AnyPin>,
        mpsc_ch: &'static InputEventChannel,
        shared_timing: &'static SharedToggleTiming,
    ) -> Self {
        let led_str: &'static str = BufferedOpenDrainKind::VendSideStartLed(player).const_str();
        let (snj_p, snj_str): (RawInputPortKind, &'static str) =
            InputPortKind::StartJam1P.to_raw_and_const_str(player);

        Self {
            in_switch: BufferedWait::new(in_switch, mpsc_ch, snj_p, snj_str),
            out_led: BufferedOpenDrain::new(out_led, shared_timing, led_str),
        }
    }

    pub fn start_tasks(&'static self, spawner: &Spawner) {
        unwrap!(spawner.spawn(buffered_wait_spawn(&self.in_switch)));
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.out_led)));
    }
}
