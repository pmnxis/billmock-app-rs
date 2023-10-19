/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use super::io_bypass::io_bypass;
use crate::boards::*;
use crate::components::eeprom;
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::input_port::{InputEvent, InputPortKind};
use crate::types::player::Player;

#[allow(dead_code)]
impl InputEvent {
    pub fn replace(&self, port: (InputPortKind, InputPortKind)) -> Self {
        if self.port == port.0 {
            Self {
                port: port.1,
                event: self.event,
            }
        } else {
            *self
        }
    }

    pub fn replace_arr(&self, ports: &[(InputPortKind, InputPortKind)]) -> Self {
        for port in ports {
            if self.port == port.0 {
                return Self {
                    port: port.1,
                    event: self.event,
                };
            }
        }
        *self
    }

    pub fn ignore(&self, port: InputPortKind) -> Self {
        // todo! - bitfield based filter system efficient instruction usage
        if self.port == port {
            Self {
                port: InputPortKind::Nothing,
                event: self.event,
            }
        } else {
            *self
        }
    }

    pub fn ignore_arr(&self, ports: &[InputPortKind]) -> Self {
        // todo! - bitfield based filter system efficient instruction usage
        for port in ports {
            if self.port == *port {
                return Self {
                    port: InputPortKind::Nothing,
                    event: self.event,
                };
            }
        }
        *self
    }

    pub fn allow(&self, port: InputPortKind) -> Self {
        // todo! - bitfield based filter system efficient instruction usage
        if self.port == port {
            *self
        } else {
            Self {
                port: InputPortKind::Nothing,
                event: self.event,
            }
        }
    }

    pub fn allow_arr(&self, ports: &[InputPortKind]) -> Self {
        // todo! - bitfield based filter system efficient instruction usage
        for port in ports {
            if self.port == *port {
                return *self;
            }
        }
        Self {
            port: InputPortKind::Nothing,
            event: self.event,
        }
    }

    pub fn flip_player(&self) -> Self {
        Self {
            port: match self.port {
                InputPortKind::Inhibit1P => InputPortKind::Inhibit2P,
                InputPortKind::Inhibit2P => InputPortKind::Inhibit1P,
                InputPortKind::Start1P => InputPortKind::Start2P,
                InputPortKind::Start2P => InputPortKind::Start1P,
                InputPortKind::Jam1P => InputPortKind::Jam2P,
                InputPortKind::Jam2P => InputPortKind::Jam1P,
                InputPortKind::StartJam1P => InputPortKind::StartJam2P,
                InputPortKind::StartJam2P => InputPortKind::StartJam1P,
                InputPortKind::Vend2P => InputPortKind::Vend1P,
                InputPortKind::Vend1P => InputPortKind::Vend2P,
                InputPortKind::Nothing => InputPortKind::Nothing,
            },
            event: self.event,
        }
    }

    pub fn ignore_player(&self, player: Player) -> Self {
        match (self.port, player) {
            (
                InputPortKind::Inhibit1P
                | InputPortKind::Jam1P
                | InputPortKind::Start1P
                | InputPortKind::Vend1P,
                Player::Player1,
            ) => *self,
            (
                InputPortKind::Inhibit2P
                | InputPortKind::Jam2P
                | InputPortKind::Start2P
                | InputPortKind::Vend2P,
                Player::Player2,
            ) => *self,
            _ => Self {
                port: InputPortKind::Nothing,
                event: self.event,
            },
        }
    }

    pub async fn apply_output(&self, board: &'static Board, override_druation_force: bool) -> Self {
        match (self.port, self.event) {
            (InputPortKind::Vend1P, InputEventKind::LongPressed(_)) => {
                let count = board
                    .hardware
                    .eeprom
                    .lock_read(eeprom::select::P1_COIN_CNT)
                    .await;
                let new_count = count + 1;

                board
                    .hardware
                    .eeprom
                    .lock_write(eeprom::select::P1_COIN_CNT, new_count)
                    .await;

                defmt::info!("P1_COIN_CNT, {} -> {}", count, new_count);
            }
            (InputPortKind::Vend2P, InputEventKind::LongPressed(_)) => {
                let count = board
                    .hardware
                    .eeprom
                    .lock_read(eeprom::select::P2_COIN_CNT)
                    .await;
                let new_count = count + 1;

                board
                    .hardware
                    .eeprom
                    .lock_write(eeprom::select::P2_COIN_CNT, new_count)
                    .await;

                defmt::info!("P2_COIN_CNT, {} -> {}", count, new_count);
            }
            _ => {}
        }

        if self.port != InputPortKind::Nothing {
            io_bypass(board, self, override_druation_force).await;
        }
        *self
    }
}
