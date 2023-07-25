/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#[allow(dead_code)]
mod nda;

use core::cell::UnsafeCell;

use defmt::info;
use embassy_stm32::peripherals::USART2;
use embassy_stm32::peripherals::{DMA1_CH1, DMA1_CH2};
use embassy_stm32::usart::{RingBufferedUartRx, UartTx};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use nda::ed785::{parse_rx, CardReaderCommand}; // NDA device proto type (not under the MIT/Apache)

const CARD_READER_COMMAND_CHANNEL_SIZE: usize = 8;

pub const CARD_READER_RX_BUFFER_SIZE: usize = 768; // most of packet is 320~330 bytes
pub const CARD_READER_TX_BUFFER_SIZE: usize = 128; // most of pakcet is 6~12 bytes, but some uncommon command can be long

pub type CardReaderCommandChannel =
    Channel<ThreadModeRawMutex, CardReaderCommand, CARD_READER_COMMAND_CHANNEL_SIZE>;

pub struct CardReaderDevice {
    // USART is complex to use generic
    tx: UnsafeCell<UartTx<'static, USART2, DMA1_CH2>>,
    rx: UnsafeCell<RingBufferedUartRx<'static, USART2, DMA1_CH1>>, // USART is complex to use generic
    channel: CardReaderCommandChannel,
}

impl CardReaderDevice {
    pub const fn new(
        tx: UartTx<'static, USART2, DMA1_CH2>,
        ringbuffer_rx: RingBufferedUartRx<'static, USART2, DMA1_CH1>,
    ) -> Self {
        Self {
            tx: UnsafeCell::new(tx),
            rx: UnsafeCell::new(ringbuffer_rx),
            channel: Channel::new(),
        }
    }

    pub async fn run(&self) {
        let rx = unsafe { &mut *self.rx.get() };
        let _tx = unsafe { &mut *self.tx.get() };
        let mut rx_buf = [0u8; CARD_READER_RX_BUFFER_SIZE];
        let mut _tx_buf = [0u8; CARD_READER_TX_BUFFER_SIZE];

        loop {
            match rx.read(&mut rx_buf).await {
                Ok(rx_len) => {
                    // for debug
                    let cutted_rx_buf = &rx_buf[..rx_len];
                    defmt::debug!("UART READ {}: {:02X}", rx_len, &cutted_rx_buf);
                    // end of debug

                    let command = parse_rx(&rx_buf, rx_len);
                    match command.coin_count() {
                        Some((c, d)) => {
                            info!("CardReaderDevice - coin_cnt:{}clk, duration:{}ms", c, d);
                        }
                        None => {}
                    }
                    self.channel.send(command).await;
                }
                Err(e) => {
                    defmt::error!("USART error : {:?}", e);
                }
            };
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
