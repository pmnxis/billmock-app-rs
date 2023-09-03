/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use serial_arcade_pay::*;

use super::{DEFAULT_BUSY_ALPHA_TIMING_MS, DEFAULT_VEND_INDICATOR_TIMING_MS};
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

    pub async fn apply_output(self, board: &'static Board, override_druation_force: bool) -> Self {
        match self.recv {
            GenericPaymentRecv::Income(GenericIncomeInfo {
                player: None,
                price: None,
                signal_count: Some(c),
                pulse_duration: Some(d),
            }) => {
                let (vend, busy, led) = self.origin.to_vend_busy_led(board);
                let coin_cnt = c.min(u8::MAX.into()) as u8;
                if override_druation_force {
                    vend.tick_tock(coin_cnt).await;
                    led.alt_tick_tock(
                        coin_cnt,
                        DEFAULT_VEND_INDICATOR_TIMING_MS,
                        DEFAULT_VEND_INDICATOR_TIMING_MS,
                    )
                    .await;
                } else {
                    busy.one_shot_high_mul(coin_cnt, d, d, DEFAULT_BUSY_ALPHA_TIMING_MS)
                        .await;
                    vend.alt_tick_tock(coin_cnt, d, d).await;
                    led.alt_tick_tock(coin_cnt, d, d).await;
                }
            }
            GenericPaymentRecv::Income(GenericIncomeInfo {
                player: Some(p),
                price: _,
                // price: Some(_r),
                signal_count: Some(c),
                pulse_duration: _,
                // pulse_duration: Some(_d),
            }) => {
                let (vend, busy, led) = match p {
                    2 => Player::Player2,
                    _ => Player::Player1,
                }
                .to_vend_busy_led(board);

                let coin_cnt = c.min(u8::MAX.into()) as u8;

                // There's no easy way to get vend timing with busy, and utilize it with one shot logic
                let timing = vend.get_shared_timing();

                busy.one_shot_high_mul(
                    coin_cnt,
                    timing.high_ms,
                    timing.low_ms,
                    DEFAULT_BUSY_ALPHA_TIMING_MS,
                )
                .await;
                vend.tick_tock(coin_cnt).await;
                led.alt_tick_tock(
                    coin_cnt,
                    DEFAULT_VEND_INDICATOR_TIMING_MS,
                    DEFAULT_VEND_INDICATOR_TIMING_MS,
                )
                .await;
            }
            _ => {}
        }

        self
    }
}
