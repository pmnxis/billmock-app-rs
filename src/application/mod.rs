/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

mod io_bypass;
mod io_card;
mod io_remap;
mod mutual_inhibit;
mod player_to_vend_led;

use embassy_futures::yield_now;
use embassy_time::Duration;
use embassy_time::Timer;
use io_card::PaymentReceive;
use serial_arcade_pay::{GenericIncomeInfo, GenericPaymentRecv};

use self::mutual_inhibit::MutualInhibit;
use crate::boards::*;
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::dip_switch_config::{AppMode0V3, TimingOverride};
use crate::types::input_port::{InputEvent, InputPortKind};
use crate::types::player::Player;

pub const DEFAULT_VEND_INDICATOR_TIMING_MS: u16 = 200;

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
        let mut appmode = AppMode0V3::default();
        let mut default_serial = Player::Undefined;
        let mut income_backup: Option<GenericIncomeInfo> = None; // for StartButtonDecideSerialToVend
        let mut mutual_inhibit = MutualInhibit::new();

        loop {
            // timing flag would be used in future implemenation.
            // reading dipsw will be changed to actor model
            let (inhibit_latest, timing_latest, appmode_latest) = hardware.dipsw.read();

            // Inhibit Override
            let prev_inhibit = mutual_inhibit.get_dipsw();
            if inhibit_latest != prev_inhibit {
                // let changed = inhibit_latest.check_changed(&prev_inhibit);
                defmt::info!("Inhibit DIP status chagned : {}", inhibit_latest);

                mutual_inhibit.update_dipsw(inhibit_latest);
                mutual_inhibit.test_and_apply_output(board).await;
            }

            // Timing Override
            if timing_latest != timing {
                let new_timing = timing_latest.get_toggle_timing();
                defmt::info!("Timing status chagned : {}", timing_latest);

                shared.arcade_players_timing[PLAYER_1_INDEX].set(new_timing);
                shared.arcade_players_timing[PLAYER_2_INDEX].set(new_timing);

                timing = timing_latest;
            }

            // AppMode setting
            if appmode_latest != appmode {
                defmt::info!("App Mode (0v3) status chagned : {}", appmode_latest);

                default_serial = match appmode_latest {
                    AppMode0V3::BypassStart | AppMode0V3::StartButtonDecideSerialToVend => {
                        Player::Undefined
                    }
                    AppMode0V3::BypassJam | AppMode0V3::BypassJamButReserved => Player::Player1,
                };

                // should be reset
                income_backup = None;

                appmode = appmode_latest;
            }

            if let Ok(x) = hardware.card_reader.recv_channel.try_receive() {
                match (
                    appmode,
                    income_backup.is_some(),
                    PaymentReceive::from((default_serial, x)),
                ) {
                    (
                        AppMode0V3::StartButtonDecideSerialToVend,
                        true,
                        PaymentReceive {
                            origin: Player::Undefined,
                            recv: GenericPaymentRecv::Income(_payment),
                        },
                    ) => {
                        defmt::warn!("StartButtonDecideSerialToVend - duplicated income received, player should press start button.");
                        hardware.card_reader.send_nack().await; // even send nack, it doesn't cancel payment with NDA device.
                    }
                    (
                        AppMode0V3::StartButtonDecideSerialToVend,
                        false,
                        PaymentReceive {
                            origin: Player::Undefined,
                            recv: GenericPaymentRecv::Income(payment),
                        },
                    ) => {
                        defmt::info!("StartButtonDecideSerialToVend - income received, wait for start button");
                        income_backup = Some(payment);
                        hardware.card_reader.send_ack().await;
                    }
                    (_, _, packet) => {
                        packet
                            .override_player_by_duration()
                            .apply_output(board, timing.is_override_force())
                            .await;
                        hardware.card_reader.send_ack().await;
                    }
                }
            }

            let input_event = if let Ok(raw_input_event) = async_input_event_ch.try_receive() {
                // let input_bits = async_input_event_ch.get_cache();
                // defmt::info!("Input cache state changed : {:04X}", input_bits);

                match InputEvent::try_from(raw_input_event) {
                    Ok(y) => {
                        let ret = y
                            .replace_arr(match appmode {
                                AppMode0V3::BypassStart
                                | AppMode0V3::StartButtonDecideSerialToVend => &[
                                    (InputPortKind::StartJam1P, InputPortKind::Start1P),
                                    (InputPortKind::StartJam2P, InputPortKind::Start2P),
                                ],
                                AppMode0V3::BypassJam | AppMode0V3::BypassJamButReserved => &[
                                    (InputPortKind::StartJam1P, InputPortKind::Jam1P),
                                    (InputPortKind::StartJam2P, InputPortKind::Jam2P),
                                ],
                            })
                            .test_mut_inh_early_output(&mut mutual_inhibit, board)
                            .await
                            .apply_output(board, timing.is_override_force())
                            .await;

                        Some(ret)
                    }
                    Err(e) => {
                        defmt::error!("Some wrong value incomed 0x{:02X}", e.number);
                        None
                    }
                }
            } else {
                None
            };

            // StartButtonDecideSerialToVend related
            if let Some((player, income, p_idx, ms)) = match (income_backup, input_event) {
                (
                    Some(income),
                    Some(InputEvent {
                        port: InputPortKind::Start1P,
                        event: InputEventKind::LongPressed(ms),
                    }),
                ) => Some((Player::Player1, income, PLAYER_1_INDEX, ms as u16)),
                (
                    Some(income),
                    Some(InputEvent {
                        port: InputPortKind::Start2P,
                        event: InputEventKind::LongPressed(ms),
                    }),
                ) => Some((Player::Player2, income, PLAYER_2_INDEX, ms as u16)),
                _ => None,
            } {
                defmt::info!(
                    "StartButtonDecideSerialToVend - condition matched, player : {}, income : {}",
                    player,
                    income,
                );

                PaymentReceive {
                    origin: player,
                    recv: GenericPaymentRecv::Income(income),
                }
                .apply_output(board, timing.is_override_force())
                .await;

                Timer::after(Duration::from_millis(500)).await;

                hardware.host_sides[p_idx]
                    .out_start
                    .alt_forever_blink(ms, ms)
                    .await;

                // should be reset
                income_backup = None;

                defmt::info!("StartButtonDecideSerialToVend - exit trigger");
            }

            yield_now().await;
        }
    }
}
