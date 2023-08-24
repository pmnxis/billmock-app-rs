/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

mod io_bypass;
mod io_card;
mod io_remap;

use embassy_futures::yield_now;
use io_card::PaymentReceive;

use crate::boards::*;
use crate::types::dip_switch_config::{AppMode0V3, InhibitOverride};
use crate::types::input_port::{InputEvent, InputPortKind};
use crate::types::player::Player;

pub struct Application {
    /// Hardware and necessary shared object
    pub board: &'static Board,
    // some service logic related
}

impl Application {
    pub fn new(board: &'static Board) -> Self {
        Self { board }
    }

    pub async fn main_task(&self) -> ! {
        // this function should be inside of loop.
        let board = self.board;
        let hardware = &self.board.hardware;
        let shared = self.board.shared_resource;
        let async_input_event_ch = &shared.async_input_event_ch;

        loop {
            // timing flag would be used in future implemenation.
            // reading dipsw will be changed to actor model
            let (inhibit_flag, _timing_flag, appmode_flag) = hardware.dipsw.read();
            let default_serial = match appmode_flag {
                AppMode0V3::BypassStart | AppMode0V3::StartButtonDecideSerialToVend => {
                    Player::Undefined
                }
                AppMode0V3::BypassJam | AppMode0V3::BypassJamAndExtraSerialPayment => {
                    Player::Player1
                }
            };

            if let Ok(x) = hardware.card_reader.channel.try_recv() {
                PaymentReceive::from((default_serial, x))
                    .override_player_by_duration()
                    .apply_output(board)
                    .await;

                // todo! - ACK pass to TX
            }

            if let Ok(raw_input_event) = async_input_event_ch.try_recv() {
                let input_bits = async_input_event_ch.get_cache();
                defmt::info!("Input cache state changed : {:04X}", input_bits);

                match InputEvent::try_from(raw_input_event) {
                    Ok(y) => {
                        y.replace_arr(match appmode_flag {
                            AppMode0V3::BypassStart | AppMode0V3::StartButtonDecideSerialToVend => {
                                &[
                                    (InputPortKind::StartJam1P, InputPortKind::Start1P),
                                    (InputPortKind::StartJam2P, InputPortKind::Start2P),
                                ]
                            }
                            AppMode0V3::BypassJam | AppMode0V3::BypassJamAndExtraSerialPayment => {
                                &[
                                    (InputPortKind::StartJam1P, InputPortKind::Jam1P),
                                    (InputPortKind::StartJam2P, InputPortKind::Jam2P),
                                ]
                            }
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
                    Err(e) => {
                        defmt::error!("Some wrong value incomed 0x{:02X}", e.number);
                    }
                }
            }

            yield_now().await;
        }
    }
}
