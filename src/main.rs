/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_main]
#![no_std]
#![feature(const_trait_impl)]
#![feature(async_fn_in_trait)]
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
use crate::components::eeprom::NovellaInitOk;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize necessary BSP
    let board: &'static mut Board = make_static!(Board::init());

    // Eeprom Novella module init
    match board.hardware.eeprom.init(false).await {
        Ok(crate::components::eeprom::NovellaInitOk::FirstBoot) => {
            defmt::info!("Welcom first boot");
        }
        Ok(crate::components::eeprom::NovellaInitOk::PartialSucess(x, y)) => {
            defmt::error!("Novella Ok But : {}, {}", x, y);
        }
        Err(crate::components::eeprom::NovellaInitError::FirstBoot) => {
            defmt::error!("FirstBoot");
        }
        Err(crate::components::eeprom::NovellaInitError::MissingEeprom) => {
            defmt::error!("MissingEeprom");
        }
        Ok(crate::components::eeprom::NovellaInitOk::Success(_)) => {
            defmt::info!("Eeprom is good status");
        }
        _ => {
            defmt::info!("mmmH?");
        }
    };

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
