/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use bit_field::BitField;
use serial_arcade_pay::TinyGenericInhibitInfo;

use crate::boards::{Board, PLAYER_1_INDEX, PLAYER_2_INDEX};
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::dip_switch_config::InhibitOverride;
use crate::types::input_port::{InputEvent, InputPortKind};
use crate::types::player::Player;

/// Mutual Inhibit module for resolve complex inhibit input / output source.
#[derive(Clone, Copy)]
pub struct MutualInhibit(u8);

impl MutualInhibit {
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub fn update_dipsw(&mut self, dipsw: InhibitOverride) {
        self.0 = (self.0 & 0b1111_1100) | ((dipsw as u8) & 0b0000_0011);
    }

    #[inline]
    pub fn get_dipsw(&self) -> InhibitOverride {
        InhibitOverride::try_from(self.0 & 0b11).unwrap() // infallable
    }

    #[inline]
    #[allow(unused)]
    pub fn update_gpio(&mut self, gpio: InhibitOverride) {
        self.0 = self.0 & 0b1111_0011 | (((gpio as u8) & 0b0000_0011) << 2);
    }

    #[inline]
    pub fn set_gpio_player(&mut self, player: Player, state: bool) {
        match player {
            Player::Undefined => {
                self.0.set_bits(2..=3, 0b11 * state as u8);
            }
            x => {
                self.0.set_bit(x as usize + 1, state);
            }
        }
    }

    #[inline]
    #[allow(unused)]
    pub fn get_gpio(&self) -> InhibitOverride {
        InhibitOverride::try_from((self.0 >> 2) & 0b11).unwrap() // infallable
    }

    pub fn test_and_check(mut self) -> Option<InhibitOverride> {
        let cmp = ((self.0 >> 2) & 0b11) | (self.0 & 0b11);
        let prev = (self.0 >> 4) & 0b11;
        if cmp != prev {
            let after = InhibitOverride::try_from(cmp).unwrap(); // infallable
            self.0 = (self.0 & 0b1100_1111) | (cmp << 4);
            Some(after)
        } else {
            None
        }
    }

    pub async fn test_and_apply_output(&mut self, board: &Board) {
        let inhibit_1p = &board.hardware.vend_sides[PLAYER_1_INDEX].out_inhibit;
        let inhibit_2p = &board.hardware.vend_sides[PLAYER_2_INDEX].out_inhibit;
        let serial_credit = &board.hardware.card_reader;

        let result = self.test_and_check();

        if let Some(x) = result {
            let (p1, p2) = ((x as u8).get_bit(0), (x as u8).get_bit(1));

            inhibit_1p.set_level(p1).await;
            inhibit_2p.set_level(p2).await;

            serial_credit
                .send_inhibit(TinyGenericInhibitInfo::new(p1, p2, false, false))
                .await;
        }
    }
}

fn map_event_kind(evt: &InputEventKind) -> bool {
    match evt {
        InputEventKind::Pressed => true,
        InputEventKind::Released => false,
        _ => false,
    }
}

impl InputEvent {
    #[allow(unused)]
    pub fn test_mut_inh(&self, mut_inh: &mut MutualInhibit) -> Self {
        if let Some((player, status)) = {
            if matches!(self.event, InputEventKind::LongPressed(_)) {
                None
            } else if self.port == InputPortKind::Inhibit1P {
                Some((Player::Player1, map_event_kind(&self.event)))
            } else if self.port == InputPortKind::Inhibit2P {
                Some((Player::Player2, map_event_kind(&self.event)))
            } else {
                None
            }
        } {
            mut_inh.set_gpio_player(player, status);
        }

        *self
    }

    pub fn test_mut_inh_then_ignore(&self, mut_inh: &mut MutualInhibit) -> Self {
        if let Some((player, status)) = {
            if matches!(self.event, InputEventKind::LongPressed(_)) {
                None
            } else if self.port == InputPortKind::Inhibit1P {
                Some((Player::Player1, map_event_kind(&self.event)))
            } else if self.port == InputPortKind::Inhibit2P {
                Some((Player::Player2, map_event_kind(&self.event)))
            } else {
                None
            }
        } {
            mut_inh.set_gpio_player(player, status);
            Self {
                port: InputPortKind::Nothing,
                event: self.event,
            }
        } else {
            *self
        }
    }

    pub async fn test_mut_inh_early_output(
        &self,
        mut_inh: &mut MutualInhibit,
        board: &Board,
    ) -> Self {
        if let Some((player, status)) = {
            if matches!(self.event, InputEventKind::LongPressed(_)) {
                None
            } else if self.port == InputPortKind::Inhibit1P {
                Some((Player::Player1, map_event_kind(&self.event)))
            } else if self.port == InputPortKind::Inhibit2P {
                Some((Player::Player2, map_event_kind(&self.event)))
            } else {
                None
            }
        } {
            mut_inh.set_gpio_player(player, status);

            mut_inh.test_and_apply_output(board).await;

            Self {
                port: InputPortKind::Nothing,
                event: self.event,
            }
        } else {
            *self
        }
    }
}
