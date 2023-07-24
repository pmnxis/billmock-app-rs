/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

// use embassy_embedded_hal::{adapter::BlockingAsync, adapter::YieldingAsync, SetConfig};
// use embassy_stm32::usart::BufferedUart;
// use embedded_hal_async::serial::*;

mod nda;

use core::cell::UnsafeCell;

use defmt::info;
use embassy_stm32::peripherals::USART2;
use embassy_stm32::usart::BufferedUart;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embedded_io::asynch::Read;
use nda::ed785::{parse_rx, CardReaderCommand}; // NDA device proto type (not under the MIT/Apache)

const CARD_READER_COMMAND_CHANNEL_SIZE: usize = 8;

pub const CARD_READER_RX_BUFFER_SIZE: usize = 768; // most of packet is 320~330 bytes
pub const CARD_READER_TX_BUFFER_SIZE: usize = 128; // most of pakcet is 6~12 bytes, but some uncommon command can be long

pub type CardReaderCommandChannel =
    Channel<ThreadModeRawMutex, CardReaderCommand, CARD_READER_COMMAND_CHANNEL_SIZE>;

pub struct CardReaderDevice {
    usart: UnsafeCell<BufferedUart<'static, USART2>>, // USART is complex to use generic
    channel: CardReaderCommandChannel,
}

impl CardReaderDevice {
    pub const fn new(usart2: BufferedUart<'static, USART2>) -> Self {
        Self {
            usart: UnsafeCell::new(usart2),
            channel: Channel::new(),
        }
    }

    pub async fn run(&self) {
        let usart = unsafe { &mut *self.usart.get() };
        let mut rx_buf = [0u8; CARD_READER_RX_BUFFER_SIZE];
        loop {
            match usart.read(&mut rx_buf).await {
                Ok(len) => {
                    let command = parse_rx(&rx_buf, len);
                    info!("Credit card Data Comes");
                    self.channel.send(command).await;
                }
                Err(e) => {
                    defmt::error!("USART error : {:?}", e);
                }
            }
        }
    }
}

// in HW v0.2 pool usage would be 1.
// single task pool consume 864 bytes
// instance include usart without dma buffer consume 28 bytes
#[embassy_executor::task(pool_size = 1)]
pub async fn card_reader_device_spawn(instance: &'static CardReaderDevice) {
    instance.run().await
}
