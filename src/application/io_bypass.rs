/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use defmt::{error, warn};

use crate::boards::*;
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::input_port::{InputEvent, InputPortKind};

pub enum VendSignalTiming {
    Player1(u16),
    Player2(u16),
}

pub async fn io_bypass(board: &'static Board, event: &InputEvent) {
    let hardware = &board.hardware;
    let shared = board.shared_resource;
    let vend_1p = &hardware.vend_sides[PLAYER_1_INDEX];
    let vend_2p = &hardware.vend_sides[PLAYER_2_INDEX];
    let host_1p = &hardware.host_sides[PLAYER_1_INDEX];
    let host_2p = &hardware.host_sides[PLAYER_2_INDEX];

    let output = match board.correspond_output(&event.port) {
        Ok(x) => x,
        Err(e) => {
            error!(
                "io_bypass some wrong enum value income : 0x{:02X}",
                e.origin
            );
            return;
        }
    };

    match (event.port, event.event) {
        (x, InputEventKind::LongPressed(0) | InputEventKind::LongPressed(1)) => {
            warn!("{:?} too short pressed", x);
        }
        (InputPortKind::Vend1P | InputPortKind::Vend2P, InputEventKind::LongPressed(x)) => {
            output.tick_tock(1).await;
        }
        (InputPortKind::StartJam1P | InputPortKind::StartJam2P, _) => {
            // skip
        }
        (_, InputEventKind::Pressed) => {
            output.set_high().await;
        }
        (_, InputEventKind::Released) => {
            output.set_low().await;
        }
        (_, InputEventKind::LongPressed(_)) => {
            // skip
        }
    }
}
