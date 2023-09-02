/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use crate::boards::{Board, LED_1_INDEX, LED_2_INDEX, PLAYER_1_INDEX, PLAYER_2_INDEX};
use crate::semi_layer::buffered_opendrain::BufferedOpenDrain;
use crate::types::player::Player;

impl Player {
    pub const fn to_vend_and_led(
        self,
        board: &'static Board,
    ) -> (&BufferedOpenDrain, &BufferedOpenDrain) {
        match self {
            Player::Player2 => (
                &board.hardware.host_sides[PLAYER_2_INDEX].out_vend,
                &board.hardware.indicators[LED_2_INDEX],
            ),
            _ => (
                &board.hardware.host_sides[PLAYER_1_INDEX].out_vend,
                &board.hardware.indicators[LED_1_INDEX],
            ),
        }
    }
}
