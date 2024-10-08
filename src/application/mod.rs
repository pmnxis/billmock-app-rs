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
mod pulse_meory_filter;

use card_terminal_adapter::types::*;
use card_terminal_adapter::*;
use embassy_futures::yield_now;
use embassy_time::{Duration, Timer};
use io_card::PaymentReceive;
#[cfg(feature = "svc_button")]
use {crate::components::eeprom, embassy_time::Instant};

use self::{mutual_inhibit::MutualInhibit, pulse_meory_filter::PulseMemoryFilterMachine};
use crate::boards::*;
use crate::semi_layer;
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::dip_switch_config::{AppMode0V3, TimingOverride};
use crate::types::input_port::{InputEvent, InputPortKind};
use crate::types::player::Player;

pub const DEFAULT_VEND_INDICATOR_TIMING_MS: u16 = 200;
pub const DEFAULT_BUSY_ALPHA_TIMING_MS: u16 = 10;

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
        let card_reader = &hardware.card_reader;
        let shared = self.board.shared_resource;
        let async_input_event_ch = &shared.async_input_event_ch;
        let mut timing = TimingOverride::default();
        let mut appmode = AppMode0V3::default();
        let mut default_serial = Player::Undefined;
        let mut income_backup: Option<PaymentReceive> = None; // for StartButtonDecideSerialToVend
        let mut mutual_inhibit = MutualInhibit::new();
        let mut did_we_ask: u8 = 0;
        let mut did_we_alert_version_warning = false;
        let mut did_we_received_slot_info = false;
        let mut slot_info_asked_time = Instant::now();
        let mut filter_state = PulseMemoryFilterMachine::new();
        #[cfg(feature = "svc_button")]
        let (mut last_svc_pressed, mut is_svc_pressed): (Instant, bool) = (Instant::now(), false);
        #[cfg(feature = "svc_button")]
        let eeprom = &hardware.eeprom;

        // Show HW info when update firmware using SWD directly
        card_reader.send(CardTerminalTxCmd::DisplayHwInfo).await;

        loop {
            // timing flag would be used in future implementation.
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
                    AppMode0V3::BypassStart
                    | AppMode0V3::StartButtonDecideSerialToVend
                    | AppMode0V3::DisplayRom => Player::Undefined,
                    AppMode0V3::BypassJam => Player::Player1,
                };

                // should be reset
                income_backup = None;

                if appmode_latest == AppMode0V3::DisplayRom {
                    card_reader.send(CardTerminalTxCmd::DisplayRom).await;
                } else if appmode == AppMode0V3::DisplayRom {
                    card_reader.send(CardTerminalTxCmd::DisplayHwInfo).await;
                }

                appmode = appmode_latest;
            }

            // Try to receive slot_info (but we need to modify this routine later)
            // internal slot info is not guarantee correctness
            // thus try to receive again on init
            if did_we_ask != 0 && !did_we_received_slot_info {
                let now = Instant::now();
                if (slot_info_asked_time + Duration::from_secs(5)) < now {
                    slot_info_asked_time = now;

                    card_reader
                        .send(CardTerminalTxCmd::RequestSaleSlotInfo)
                        .await;

                    if !board
                        .hardware
                        .eeprom
                        .lock_read(eeprom::select::CARD_PORT_BACKUP)
                        .await
                        .is_zeroed()
                    {
                        did_we_received_slot_info = true;
                    }
                }
            }

            if let Ok(x) = card_reader.recv_channel.try_receive() {
                match x {
                    CardTerminalRxCmd::RequestDeviceInfo => {
                        did_we_ask = did_we_ask.checked_add(1).unwrap_or(u8::MAX);

                        card_reader
                            .send(CardTerminalTxCmd::ResponseDeviceInfo)
                            .await;

                        defmt::info!("Card Terminal asked {} times", did_we_ask);

                        // Allow wait few times, because
                        Timer::after(Duration::from_millis(100)).await;

                        card_reader
                            .send(CardTerminalTxCmd::RequestTerminalInfo)
                            .await;

                        Timer::after(Duration::from_millis(500)).await;

                        card_reader
                            .send(CardTerminalTxCmd::RequestSaleSlotInfo)
                            .await;

                        slot_info_asked_time = Instant::now();
                    }
                    CardTerminalRxCmd::AlertPaymentIncomeArcade(raw_income) => {
                        // judge current application mode and income backup
                        let payment = PaymentReceive::from((default_serial, raw_income.into()));

                        if appmode == AppMode0V3::StartButtonDecideSerialToVend {
                            match income_backup.is_some() {
                                true => {
                                    defmt::warn!("StartButtonDecideSerialToVend - duplicated income received, player should press start button.");
                                    card_reader.send_nack().await; // even send nack, it doesn't cancel payment with NDA device.
                                }
                                false => {
                                    defmt::info!("StartButtonDecideSerialToVend - income received, wait for start button");
                                    income_backup = Some(payment);
                                    card_reader.send_ack().await;
                                }
                            }
                        } else {
                            payment
                                // .override_player_by_duration()
                                .apply_output(board, timing.is_override_force())
                                .await;
                            card_reader.send_ack().await;
                        }
                    }
                    CardTerminalRxCmd::AlertPaymentIncomePrice(raw_price) => {
                        let u32_price: u32 = raw_price.into();

                        let payment = PaymentReceive::from((
                            default_serial,
                            IncomeArcadeRequest {
                                port: 0, // fake value,
                                pulse_count: (u32_price / 500).max(1).max(u8::MAX as u32) as u16,
                                pulse_duration: semi_layer::timing::ToggleTiming::default().high_ms,
                            },
                        ));

                        if appmode == AppMode0V3::StartButtonDecideSerialToVend {
                            match income_backup.is_some() {
                                true => {
                                    defmt::warn!("StartButtonDecideSerialToVend - duplicated income received, player should press start button.");
                                    card_reader.send_nack().await; // even send nack, it doesn't cancel payment with NDA device.
                                }
                                false => {
                                    defmt::info!("StartButtonDecideSerialToVend - income received, wait for start button");
                                    income_backup = Some(payment);
                                    card_reader.send_ack().await;
                                }
                            }
                        } else {
                            payment
                                // .override_player_by_duration()
                                .apply_output(board, timing.is_override_force())
                                .await;
                            card_reader.send_ack().await;
                        }
                    }
                    CardTerminalRxCmd::ResponseSaleSlotInfo => {
                        // read from lock_read for do something
                        // todo! - handle different TId/and something
                    }
                    // read from lock_read for do something
                    // handle different TID/and something
                    CardTerminalRxCmd::ResponseTerminalInfo(tid_status, terminal_ver) => {
                        if tid_status == TidStatus::Changed {
                            card_reader
                                .send(CardTerminalTxCmd::RequestSaleSlotInfo)
                                .await;
                        }

                        if !did_we_alert_version_warning {
                            if let Some(alert) = match terminal_ver {
                                TerminalVersion::ArcadeSpecificLatest => None,
                                TerminalVersion::ArcadeSpecificLegacy => {
                                    Some(CardTerminalDisplayWarning::RequireLatestTerminalVersion)
                                }
                                TerminalVersion::GenericPriceIncomeType => {
                                    Some(CardTerminalDisplayWarning::RequireArcadeSpecificVersion)
                                }
                                TerminalVersion::Experimental => {
                                    Some(CardTerminalDisplayWarning::WarnExperimentalVesion)
                                }
                                TerminalVersion::Unknown => {
                                    Some(CardTerminalDisplayWarning::WarnUnknown)
                                }
                            } {
                                card_reader
                                    .send(CardTerminalTxCmd::DisplayWarning(alert))
                                    .await;

                                did_we_alert_version_warning = true;
                            }
                        }
                    }

                    _ => {}
                }
            }

            // Arcade legacy,
            let input_event = if let Ok(raw_input_event) = async_input_event_ch.try_receive() {
                // let input_bits = async_input_event_ch.get_cache();
                // defmt::info!("Input cache state changed : {:04X}", input_bits);

                match InputEvent::try_from(raw_input_event) {
                    #[cfg(feature = "svc_button")]
                    Ok(InputEvent {
                        port: InputPortKind::SvcButton,
                        event,
                    }) => {
                        match event {
                            InputEventKind::Pressed => {
                                is_svc_pressed = true;
                                last_svc_pressed = Instant::now();
                            }
                            InputEventKind::LongPressed(t) => {
                                if t < 2 {
                                } else if (2 < t) && (t < 120) {
                                    card_reader.send(CardTerminalTxCmd::DisplayRom).await;
                                } else {
                                    // Factory reset by SvcButton
                                    // but this clear only 1/2p credit and coin count
                                    if ((Instant::now() - last_svc_pressed)
                                        > Duration::from_secs(10))
                                        && is_svc_pressed
                                    {
                                        defmt::info!("Factory reset EEPROM");

                                        card_reader
                                            .send(CardTerminalTxCmd::DisplayWarning(
                                                CardTerminalDisplayWarning::WarnEepromFactoryReset,
                                            ))
                                            .await;

                                        eeprom.lock_write_zero(eeprom::select::P1_CARD_CNT).await;
                                        eeprom.lock_write_zero(eeprom::select::P2_CARD_CNT).await;
                                        eeprom.lock_write_zero(eeprom::select::P1_COIN_CNT).await;
                                        eeprom.lock_write_zero(eeprom::select::P2_COIN_CNT).await;
                                    } else {
                                        card_reader.send(CardTerminalTxCmd::DisplayHwInfo).await;
                                    }
                                }
                                is_svc_pressed = false;
                            }
                            InputEventKind::Released => {
                                is_svc_pressed = false;
                            }
                        }

                        yield_now().await;
                        // Some(InputEvent {
                        //     port: InputPortKind::SvcButton,
                        //     event: InputEventKind::LongPressed(t),
                        // })
                        None
                    }
                    Ok(y) => {
                        let ret = y
                            .replace_arr(match appmode {
                                AppMode0V3::BypassStart
                                | AppMode0V3::StartButtonDecideSerialToVend => &[
                                    (InputPortKind::StartJam1P, InputPortKind::Start1P),
                                    (InputPortKind::StartJam2P, InputPortKind::Start2P),
                                ],
                                AppMode0V3::BypassJam | AppMode0V3::DisplayRom => &[
                                    (InputPortKind::StartJam1P, InputPortKind::Jam1P),
                                    (InputPortKind::StartJam2P, InputPortKind::Jam2P),
                                ],
                            })
                            .test_mut_inh_early_output(&mut mutual_inhibit, board)
                            .await
                            .apply_output(board, &mut filter_state, timing.is_override_force())
                            .await;

                        Some(ret)
                    }
                    Err(e) => {
                        defmt::error!("Some wrong value incomed 0x{:02X}", e.number);
                        None
                    }
                }
            } else {
                filter_state.report_when_expired(board).await;

                None
            };

            // StartButtonDecideSerialToVend related
            if let Some((player, income, p_idx, ms)) = match (income_backup.clone(), input_event) {
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
                    recv: income.recv,
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
