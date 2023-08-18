/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

mod boards;
mod components;
mod semi_layer;
mod types;

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use serial_arcade_pay::{GenericIncomeInfo, GenericPaymentRecv};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

use crate::{
    boards::*,
    semi_layer::buffered_wait::{InputEventKind, InputPortKind},
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize necessary BSP
    let board: &'static mut Board = make_static!(Board::init());

    // Spawns a task bound to the BSP
    board.start_tasks(&spawner);
    let hardware = &board.hardware;
    let shared = &board.shared_resource;
    let vend_1p = &hardware.vend_sides[PLAYER_1_INDEX];
    let vend_2p = &hardware.vend_sides[PLAYER_2_INDEX];
    let host_1p = &hardware.host_sides[PLAYER_1_INDEX];
    let host_2p = &hardware.host_sides[PLAYER_2_INDEX];

    info!("Hello BillMock");

    loop {
        match hardware.card_reader.channel.try_recv().ok() {
            Some(GenericPaymentRecv::Income(GenericIncomeInfo {
                player: None,
                price: None,
                signal_count: Some(c),
                pulse_duration: Some(d),
            })) => {
                match d % 10 != 2 {
                    true => host_1p,
                    false => host_2p,
                }
                .out_vend
                .tick_tock(c.min(u8::MAX.into()) as u8)
                .await;
            }
            Some(GenericPaymentRecv::Income(GenericIncomeInfo {
                player: Some(p),
                price: _,
                // price: Some(_r),
                signal_count: Some(c),
                pulse_duration: _,
                // pulse_duration: Some(_d),
            })) => {
                match p {
                    2 => host_2p,
                    _ => host_1p,
                }
                .out_vend
                .tick_tock(c.min(u8::MAX.into()) as u8)
                .await;
            }
            None => {}
            _ => {}
        }

        match shared.async_input_event_ch.try_recv().ok() {
            Some(event) => {
                // info!("EVENT comes - port:{}, kind:{}", event.port, event.kind);
                info!("Some event comes");
                match (event.port, InputEventKind::from(event.kind)) {
                    (InputPortKind::Start1P, InputEventKind::Pressed) => {
                        info!("Start1P Pressed");
                        host_1p.out_start.set_high().await;
                    }
                    (InputPortKind::Start1P, InputEventKind::Released) => {
                        info!("Start1P Released");
                        host_1p.out_start.set_low().await;
                    }
                    (InputPortKind::Start2P, InputEventKind::Pressed) => {
                        info!("Start2P Pressed");
                        host_2p.out_start.set_high().await;
                    }
                    (InputPortKind::Start2P, InputEventKind::Released) => {
                        info!("Start2P Released");
                        host_2p.out_start.set_low().await;
                    }
                    (InputPortKind::Inhibit1P, InputEventKind::Pressed) => {
                        vend_1p.out_inhibit.set_high().await;
                    }
                    (InputPortKind::Inhibit1P, InputEventKind::Released) => {
                        vend_2p.out_inhibit.set_low().await;
                    }
                    (InputPortKind::Vend1P, InputEventKind::LongPressed(duration_10ms)) => {
                        if duration_10ms > 1 {
                            info!("Vend LongPressed");

                            // this is proof of concept, doesn't cover all start1p/2p complex selection
                            host_1p.out_vend.tick_tock(1).await;
                        }
                    }
                    (InputPortKind::Vend1P, InputEventKind::Pressed) => {
                        info!("Vend Pressed")
                    }
                    (InputPortKind::Vend1P, InputEventKind::Released) => {
                        info!("Vend Released")
                    }
                    // not implement JAM side
                    _ => {}
                }
            }
            None => {}
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {}
