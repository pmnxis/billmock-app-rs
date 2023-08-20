/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

mod io_bypass;
mod io_remap;

use self::io_remap::InsteadStartJam;
use crate::components::dip_switch::DipSwitch;
use crate::types::dip_switch_config::{AppMode0V3, InhibitOverride};
use crate::types::input_port::{InputEvent, InputPortKind};
use crate::{boards::*, semi_layer::buffered_wait::InputEventKind};

/*

io_remap
time median capturing
inhibit override hold



*/

pub struct Application {
    /// Hardware and necessary shared object
    pub board: &'static Board,
    // some service logic related
}

impl Application {
    pub async fn stuff(&self) {
        // this function should be inside of loop.
        let board = self.board;
        let hardware = &self.board.hardware;
        let shared = self.board.shared_resource;

        loop {
            let (inhibit_flag, timing_flag, appmode_flag) = hardware.dipsw.read();

            let card_input = hardware.card_reader.channel.try_recv().ok();
            // do some work

            let mut input = match shared.async_input_event_ch.try_recv().ok() {
                Some(x) => match InputEvent::try_from(x) {
                    Ok(y) => y,
                    Err(e) => {
                        defmt::error!("Some wrong value incomed 0x{:02X}", e.number);
                        continue;
                    }
                },
                _ => {
                    continue;
                }
            };

            let _input = input
                .replace_arr(match appmode_flag {
                    AppMode0V3::BypassStart | AppMode0V3::StartButtonDecideSerialToVend => &[
                        (InputPortKind::StartJam1P, InputPortKind::Start1P),
                        (InputPortKind::StartJam2P, InputPortKind::Start2P),
                    ],
                    AppMode0V3::BypassJam | AppMode0V3::BypassJamAndExtraSerialPayment => &[
                        (InputPortKind::StartJam1P, InputPortKind::Jam1P),
                        (InputPortKind::StartJam2P, InputPortKind::Jam2P),
                    ],
                })
                .ignore_arr(match inhibit_flag {
                    InhibitOverride::Normal => &[],
                    InhibitOverride::ForceInhibit1P => &[InputPortKind::Inhibit1P],
                    InhibitOverride::ForceInhibit2P => &[InputPortKind::Inhibit2P],
                    InhibitOverride::ForceInhibitGlobal => {
                        &[InputPortKind::Inhibit1P, InputPortKind::Inhibit2P]
                    }
                })
                .apply_output(board)
                .await;
        }
    }
}
