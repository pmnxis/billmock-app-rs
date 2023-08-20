/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_main]
#![no_std]
#![feature(const_trait_impl)]
#![feature(type_alias_impl_trait)]

mod application;
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

use crate::types::input_port::{InputEvent, InputPortKind};
use crate::{boards::*, semi_layer::buffered_wait::InputEventKind};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize necessary BSP
    let board: &'static mut Board = make_static!(Board::init());

    // Spawns a task bound to the BSP
    board.start_tasks(&spawner);
    let hardware = &board.hardware;
    let shared = board.shared_resource;
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

        Timer::after(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {}
