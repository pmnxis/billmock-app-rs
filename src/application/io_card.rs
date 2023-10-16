/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use card_terminal_adapter::types::IncomeArcadeRequest;

use super::{DEFAULT_BUSY_ALPHA_TIMING_MS, DEFAULT_VEND_INDICATOR_TIMING_MS};
use crate::application::CardReaderPortBackup;
use crate::components::eeprom::{NovellaSelector, NvMemSectionKind};
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
    /// This is a hotfix intended to address the issue of certain card terminals having
    /// their player numbers not implemented correctly in the serial protocol.
    /// It enforces a solution to this problem by overriding the player number
    /// to 2 when the last digit of a decimal number ends with 2.
    // pub fn override_player_by_duration(self) -> Self {
    //     match self.recv {
    //         IncomeArcadeRequest::Income(GenericIncomeInfo {
    //             player: None,
    //             price: None,
    //             signal_count: Some(c),
    //             pulse_duration: Some(d),
    //         }) => Self {
    //             origin: self.origin,
    //             recv: IncomeArcadeRequest::Income(GenericIncomeInfo {
    //                 player: Some(match d % 10 != 2 {
    //                     true => 1,
    //                     false => 2,
    //                 }),
    //                 price: None,
    //                 signal_count: Some(c),
    //                 pulse_duration: Some(d - (d % 10)),
    //             }),
    //         },
    //         _ => self,
    //     }
    // }

    pub async fn apply_output(self, board: &'static Board, override_druation_force: bool) -> Self {
        // let player = {
        //     let selctor: NovellaSelector<CardReaderPortBackup> = NovellaSelector {
        //         section: NvMemSectionKind::CardPortBackup,
        //         marker: core::marker::PhantomData,
        //     };

        //     board
        //         .hardware
        //         .eeprom
        //         .lock_read(selctor)
        //         .await
        //         .guess_player_by_port_num(self.recv.port)
        // };

        let (vend, busy, led) = self.origin.to_vend_busy_led(board);

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

        led.alt_tick_tock(
            coin_cnt,
            DEFAULT_VEND_INDICATOR_TIMING_MS,
            DEFAULT_VEND_INDICATOR_TIMING_MS,
        )
        .await;

        // apply output

        // match self.recv {
        //     IncomeArcadeRequest::Income(GenericIncomeInfo {
        //         player: None,
        //         price: None,
        //         signal_count: Some(c),
        //         pulse_duration: Some(d),
        //     }) => {
        //         let (vend, busy, led) = self.origin.to_vend_busy_led(board);
        //         let coin_cnt = c.min(u8::MAX.into()) as u8;
        //         if override_druation_force {
        //             vend.tick_tock(coin_cnt).await;
        //             led.alt_tick_tock(
        //                 coin_cnt,
        //                 DEFAULT_VEND_INDICATOR_TIMING_MS,
        //                 DEFAULT_VEND_INDICATOR_TIMING_MS,
        //             )
        //             .await;
        //         } else {
        //             busy.one_shot_high_mul(coin_cnt, d, d, DEFAULT_BUSY_ALPHA_TIMING_MS)
        //                 .await;
        //             vend.alt_tick_tock(coin_cnt, d, d).await;
        //             led.alt_tick_tock(coin_cnt, d, d).await;
        //         }
        //     }
        //     IncomeArcadeRequest::Income(GenericIncomeInfo {
        //         player: Some(p),
        //         price: _,
        //         // price: Some(_r),
        //         signal_count: Some(c),
        //         pulse_duration: _,
        //         // pulse_duration: Some(_d),
        //     }) => {
        //         let (vend, busy, led) = match p {
        //             2 => Player::Player2,
        //             _ => Player::Player1,
        //         }
        //         .to_vend_busy_led(board);

        //         let coin_cnt = c.min(u8::MAX.into()) as u8;

        //         // There's no easy way to get vend timing with busy, and utilize it with one shot logic
        //         let timing = vend.get_shared_timing();

        //         busy.one_shot_high_mul(
        //             coin_cnt,
        //             timing.high_ms,
        //             timing.low_ms,
        //             DEFAULT_BUSY_ALPHA_TIMING_MS,
        //         )
        //         .await;
        //         vend.tick_tock(coin_cnt).await;
        //         led.alt_tick_tock(
        //             coin_cnt,
        //             DEFAULT_VEND_INDICATOR_TIMING_MS,
        //             DEFAULT_VEND_INDICATOR_TIMING_MS,
        //         )
        //         .await;
        //     }
        //     _ => {}
        // }

        self
    }
}
