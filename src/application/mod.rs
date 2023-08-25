/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

mod io_bypass;
mod io_card;
mod io_remap;

use bit_field::BitField;
use embassy_futures::yield_now;
use io_card::PaymentReceive;

use crate::boards::*;
use crate::types::dip_switch_config::{AppMode0V3, InhibitOverride, TimingOverride};
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
        let mut timing = TimingOverride::default();
        let mut inhibit = InhibitOverride::default();

        loop {
            // timing flag would be used in future implemenation.
            // reading dipsw will be changed to actor model
            let (inhibit_flag, timing_flag, appmode_flag) = hardware.dipsw.read();
            let default_serial = match appmode_flag {
                AppMode0V3::BypassStart | AppMode0V3::StartButtonDecideSerialToVend => {
                    Player::Undefined
                }
                AppMode0V3::BypassJam | AppMode0V3::BypassJamAndExtraSerialPayment => {
                    Player::Player1
                }
            };

            // Timing Override
            if timing_flag != timing {
                let new_timing = timing_flag.get_toggle_timing();

                shared.arcade_players_timing[PLAYER_1_INDEX].set(new_timing);
                shared.arcade_players_timing[PLAYER_2_INDEX].set(new_timing);

                timing = timing_flag;
            }

            // Inhibit Override
            if inhibit_flag != inhibit {
                let changed = inhibit_flag.check_changed(&inhibit);
                defmt::info!("Inhibit status chagned, P1/2 : {:?}", changed);

                if let Some(x) = changed.0 {
                    let io_status = async_input_event_ch
                        .get_cache()
                        .get_bit(usize::from(InputPortKind::Inhibit1P as u8))
                        as bool;

                    hardware.vend_sides[PLAYER_1_INDEX]
                        .out_inhibit
                        .set_level(x | io_status)
                        .await;
                    // send uart set inhibited
                }
                if let Some(x) = changed.1 {
                    let io_status = async_input_event_ch
                        .get_cache()
                        .get_bit(usize::from(InputPortKind::Inhibit2P as u8))
                        as bool;
                    hardware.vend_sides[PLAYER_2_INDEX]
                        .out_inhibit
                        .set_level(x | io_status)
                        .await;
                    // send uart set inhibited
                }

                inhibit = inhibit_flag;
            }

            if let Ok(x) = hardware.card_reader.channel.try_receive() {
                PaymentReceive::from((default_serial, x))
                    .override_player_by_duration()
                    .apply_output(board)
                    .await;

                // todo! - ACK pass to TX
            }

            if let Ok(raw_input_event) = async_input_event_ch.try_receive() {
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
                        .ignore_arr(match inhibit {
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
