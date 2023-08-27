/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_main]
#![no_std]
#![feature(const_trait_impl)]
#![feature(type_alias_impl_trait)]
#![feature(effects)] // see : https://github.com/rust-lang/rust/issues/114808

mod application;
mod boards;
mod components;
mod semi_layer;
mod types;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

use crate::application::Application;
use crate::boards::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize necessary BSP
    let board: &'static mut Board = make_static!(Board::init());

    // heuristic wait for stablize external electronic status
    Timer::after(Duration::from_millis(1000)).await;

    // Spawns a task bound to the BSP
    board.start_tasks(&spawner);

    // heuristic wait for stablize task spawning
    Timer::after(Duration::from_millis(500)).await;

    defmt::info!("Hello BillMock");

    let application = Application::new(board);
    application.main_task().await;
}

#[cfg(test)]
mod tests {}
