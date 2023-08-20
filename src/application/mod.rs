/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

mod io_bypass;
mod io_remap;

use self::io_remap::InsteadStartJam;
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
        let mut instead = InsteadStartJam::Start;

        loop {
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

            input.remap(instead);
            io_bypass::io_bypass(board, &input).await;
        }
    }
}
