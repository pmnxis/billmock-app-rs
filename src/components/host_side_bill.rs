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
use crate::semi_layer::timing::SharedToggleTiming;
use crate::types::buffered_opendrain_kind::BufferedOpenDrainKind;
use crate::types::const_convert::ConstInto;
use crate::types::input_port::InputPortKind;
use crate::types::player::Player;

pub struct HostSideBill {
    in_inhibit: BufferedWait,
    pub out_busy: BufferedOpenDrain,
    pub out_vend: BufferedOpenDrain,
    pub out_jam: BufferedOpenDrain,
    pub out_start: BufferedOpenDrain,
}

impl HostSideBill {
    pub const fn new(
        player: Player,
        in_inhibit: ExtiInput<'static, AnyPin>,
        in_inhibit_event: InputPortKind,
        out_busy: Output<'static, AnyPin>,
        out_vend: Output<'static, AnyPin>,
        out_jam: Output<'static, AnyPin>,
        out_start: Output<'static, AnyPin>,
        mpsc_ch: &'static InputEventChannel,
        shared_timing: &'static SharedToggleTiming,
    ) -> Self {
        let busy_str: &'static str = BufferedOpenDrainKind::HostSideOutBusy(player).const_str();
        let vend_str: &'static str = BufferedOpenDrainKind::HostSideOutVend(player).const_str();
        let jam_str: &'static str = BufferedOpenDrainKind::HostSideOutJam(player).const_str();
        let start_str: &'static str = BufferedOpenDrainKind::HostSideOutStart(player).const_str();

        Self {
            in_inhibit: BufferedWait::new(in_inhibit, in_inhibit_event.const_into(), mpsc_ch),
            out_busy: BufferedOpenDrain::new(out_busy, shared_timing, busy_str),
            out_vend: BufferedOpenDrain::new(out_vend, shared_timing, vend_str),
            out_jam: BufferedOpenDrain::new(out_jam, shared_timing, jam_str),
            out_start: BufferedOpenDrain::new(out_start, shared_timing, start_str),
        }
    }

    #[allow(dead_code)]
    pub async fn set_bulk_signal_all(&self, busy: bool, vend: bool, jam: bool, start: bool) {
        self.out_busy.set_level(busy).await;
        self.out_vend.set_level(vend).await;
        self.out_jam.set_level(jam).await;
        self.out_start.set_level(start).await;
    }

    pub fn start_tasks(&'static self, spawner: &Spawner) {
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.out_busy)));
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.out_vend)));
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.out_jam)));
        unwrap!(spawner.spawn(buffered_opendrain_spawn(&self.out_start)));
        unwrap!(spawner.spawn(buffered_wait_spawn(&self.in_inhibit)));
    }
}
