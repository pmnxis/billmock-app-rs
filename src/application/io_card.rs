/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use card_terminal_adapter::types::IncomeArcadeRequest;

use super::{DEFAULT_BUSY_ALPHA_TIMING_MS, DEFAULT_VEND_INDICATOR_TIMING_MS};
use crate::components::eeprom;
use crate::{boards::*, types::player::Player};

#[derive(Debug, defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PaymentReceive {
    pub origin: Player,
    pub recv: IncomeArcadeRequest,
}

impl From<(IncomeArcadeRequest, Player)> for PaymentReceive {
    fn from(value: (IncomeArcadeRequest, Player)) -> Self {
        Self {
            origin: value.1,
            recv: value.0,
        }
    }
}

impl From<(Player, IncomeArcadeRequest)> for PaymentReceive {
    fn from(value: (Player, IncomeArcadeRequest)) -> Self {
        Self {
            origin: value.0,
            recv: value.1,
        }
    }
}

impl From<IncomeArcadeRequest> for PaymentReceive {
    fn from(value: IncomeArcadeRequest) -> Self {
        Self {
            origin: Player::default(),
            recv: value,
        }
    }
}

impl PaymentReceive {
    pub async fn apply_output(self, board: &'static Board, override_druation_force: bool) -> Self {
        let player = {
            defmt::debug!("port 0x{:02X}", self.recv.port);
            let temp = (self.recv.port.max(1).min(4) - 1) & 0x1;

            match temp {
                0 => Player::Player1,
                1 => Player::Player2,
                x => {
                    defmt::warn!("Something is wrong {}", x);
                    Player::Player1
                }
            }
        };

        if player != self.origin {
            defmt::warn!(
                "mismatch player origin, origin : {:?}, player : {:?}",
                self.origin,
                player
            );
        }

        let (vend, busy, led) = player.to_vend_busy_led(board);

        let coin_cnt = self.recv.pulse_count.min(u8::MAX as u16) as u8;
        let d = self.recv.pulse_duration;

        if override_druation_force {
            vend.tick_tock(coin_cnt).await;
            busy.one_shot_high_shared_alpha(coin_cnt, DEFAULT_BUSY_ALPHA_TIMING_MS)
                .await;
        } else {
            vend.alt_tick_tock(coin_cnt, d, d).await;
            busy.one_shot_high_mul(coin_cnt, d, d, DEFAULT_BUSY_ALPHA_TIMING_MS)
                .await;
        }

        match player {
            Player::Player1 => {
                let count = board
                    .hardware
                    .eeprom
                    .lock_read(eeprom::select::P1_CARD_CNT)
                    .await;
                let new_count = count + coin_cnt as u32;

                board
                    .hardware
                    .eeprom
                    .lock_write(eeprom::select::P1_CARD_CNT, new_count)
                    .await;

                defmt::debug!("P1_CARD_CNT, {} -> {}", count, new_count);
            }
            Player::Player2 => {
                let count = board
                    .hardware
                    .eeprom
                    .lock_read(eeprom::select::P2_CARD_CNT)
                    .await;
                let new_count = count + coin_cnt as u32;

                board
                    .hardware
                    .eeprom
                    .lock_write(eeprom::select::P2_CARD_CNT, new_count)
                    .await;

                defmt::debug!("P2_CARD_CNT, {} -> {}", count, new_count);
            }
            _ => {
                defmt::error!("Unrecheable");
            }
        }

        led.alt_tick_tock(
            coin_cnt,
            DEFAULT_VEND_INDICATOR_TIMING_MS,
            DEFAULT_VEND_INDICATOR_TIMING_MS,
        )
        .await;

        self
    }
}
