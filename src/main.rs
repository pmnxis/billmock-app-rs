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
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

use crate::boards::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize necessary BSP
    let board: &'static mut Board = make_static!(Board::init());

    // Spawns a task bound to the BSP
    board.start_tasks(&spawner);

    info!("Hello BillMock");

    loop {
        // write event based business logic here.

        Timer::after(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {}
