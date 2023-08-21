/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use serial_arcade_pay::*;

use crate::{boards::*, types::player::Player};

#[derive(Debug, defmt::Format, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PaymentReceive {
    pub origin: Player,
    pub recv: GenericPaymentRecv,
}

impl From<(GenericPaymentRecv, Player)> for PaymentReceive {
    fn from(value: (GenericPaymentRecv, Player)) -> Self {
        Self {
            origin: value.1,
            recv: value.0,
        }
    }
}

impl From<(Player, GenericPaymentRecv)> for PaymentReceive {
    fn from(value: (Player, GenericPaymentRecv)) -> Self {
        Self {
            origin: value.0,
            recv: value.1,
        }
    }
}

impl From<GenericPaymentRecv> for PaymentReceive {
    fn from(value: GenericPaymentRecv) -> Self {
        Self {
            origin: Player::default(),
            recv: value,
        }
    }
}

impl PaymentReceive {
    /// This is a hotfix intended to address the issue of certain card terminals having
    /// their player numbers not implemented correctly in the serial protocol.
    /// It enforces a solution to this problem by overriding the player number
    /// to 2 when the last digit of a decimal number ends with 2.
    pub fn override_player_by_duration(self) -> Self {
        match self.recv {
            GenericPaymentRecv::Income(GenericIncomeInfo {
                player: None,
                price: None,
                signal_count: Some(c),
                pulse_duration: Some(d),
            }) => Self {
                origin: self.origin,
                recv: GenericPaymentRecv::Income(GenericIncomeInfo {
                    player: Some(match d % 10 != 2 {
                        true => 1,
                        false => 2,
                    }),
                    price: None,
                    signal_count: Some(c),
                    pulse_duration: Some(d - (d % 10)),
                }),
            },
            _ => self,
        }
    }

    pub async fn apply_output(self, board: &'static Board) -> Self {
        let hardware = &board.hardware;
        let host_1p = &hardware.host_sides[PLAYER_1_INDEX];
        let host_2p = &hardware.host_sides[PLAYER_2_INDEX];

        match self.recv {
            GenericPaymentRecv::Income(GenericIncomeInfo {
                player: None,
                price: None,
                signal_count: Some(c),
                pulse_duration: Some(_d),
            }) => {
                match self.origin {
                    Player::Player1 => host_1p,
                    Player::Player2 => host_2p,
                    _ => host_1p,
                }
                .out_vend
                .tick_tock(c.min(u8::MAX.into()) as u8)
                .await;
            }
            GenericPaymentRecv::Income(GenericIncomeInfo {
                player: Some(p),
                price: _,
                // price: Some(_r),
                signal_count: Some(c),
                pulse_duration: _,
                // pulse_duration: Some(_d),
            }) => {
                match p {
                    2 => host_2p,
                    _ => host_1p,
                }
                .out_vend
                .tick_tock(c.min(u8::MAX.into()) as u8)
                .await;
            }
            _ => {}
        }

        self
    }
}
