/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use core::cell::UnsafeCell;

use embassy_stm32::peripherals::USART2;
use embassy_stm32::peripherals::{DMA1_CH1, DMA1_CH2};
use embassy_stm32::usart::{RingBufferedUartRx, UartTx};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use serial_arcade_pay::*;
use serial_arcade_pay_impl::SerialPayVarient;

const CARD_READER_COMMAND_CHANNEL_SIZE: usize = 8;

pub const CARD_READER_RX_BUFFER_SIZE: usize = 768; // most of packet is 320~330 bytes
pub const CARD_READER_TX_BUFFER_SIZE: usize = 128; // most of pakcet is 6~12 bytes, but some uncommon command can be long

pub type CardReaderResponseChannel =
    Channel<ThreadModeRawMutex, GenericPaymentRecv, CARD_READER_COMMAND_CHANNEL_SIZE>;

pub type CardReaderRequestChannel =
    Channel<ThreadModeRawMutex, GenericPaymentRequest, CARD_READER_COMMAND_CHANNEL_SIZE>;

pub struct CardReaderDevice {
    // USART is complex to use generic
    tx: UnsafeCell<UartTx<'static, USART2, DMA1_CH2>>,
    rx: UnsafeCell<RingBufferedUartRx<'static, USART2, DMA1_CH1>>, // USART is complex to use generic
    pub recv_channel: CardReaderResponseChannel,
    pub req_channel: CardReaderRequestChannel,
}

impl CardReaderDevice {
    pub const fn new(
        tx: UartTx<'static, USART2, DMA1_CH2>,
        ringbuffer_rx: RingBufferedUartRx<'static, USART2, DMA1_CH1>,
    ) -> Self {
        Self {
            tx: UnsafeCell::new(tx),
            rx: UnsafeCell::new(ringbuffer_rx),
            recv_channel: Channel::new(),
            req_channel: Channel::new(),
        }
    }

    pub async fn run(&self) {
        let mut current_varient: Option<SerialPayVarient> = None;

        let rx = unsafe { &mut *self.rx.get() };
        let tx = unsafe { &mut *self.tx.get() };
        let mut rx_buf = [0u8; CARD_READER_RX_BUFFER_SIZE];
        let mut tx_buf = [0u8; CARD_READER_TX_BUFFER_SIZE];

        loop {
            // TX not hang on IO wait
            if let (Some(varient), Ok(request)) = (current_varient, self.req_channel.try_receive())
            {
                match varient.generate_tx(&request, &mut tx_buf) {
                    Ok(len) => {
                        if let Err(e_dma) = tx.write(&tx_buf[0..len]).await {
                            defmt::error!("USART TX error : {:?}", e_dma);
                        }
                    }
                    Err(e) => {
                        defmt::error!("GEN TX error : {:?}", e);
                    }
                }
            }

            // RX hang IO wait
            match rx.read(&mut rx_buf).await {
                Ok(rx_len) => {
                    // for debug
                    let cutted_rx_buf = &rx_buf[..rx_len];
                    defmt::debug!("UART READ {}: {:02X}", rx_len, &cutted_rx_buf);
                    // end of debug

                    match SerialPayVarient::parse_rx(&rx_buf, rx_len) {
                        Ok((resp, varient)) => {
                            self.recv_channel.send(resp).await;
                            // todo! - generic unknown checker
                            current_varient = Some(varient);
                        }
                        Err(_e) => { /* parse error */ }
                    }
                }
                Err(e) => {
                    defmt::error!("USART error : {:?}", e);
                }
            };
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub async fn send(&self, request: GenericPaymentRequest) {
        self.req_channel.send(request).await;
    }

    pub async fn send_ack(&self) {
        self.req_channel.send(GenericPaymentRequest::Ack).await;
    }

    pub async fn send_nack(&self) {
        self.req_channel.send(GenericPaymentRequest::Nack).await;
    }

    pub async fn send_inhibit(&self, inhibit: TinyGenericInhibitInfo) {
        self.req_channel
            .send(GenericPaymentRequest::SetInhibit(inhibit))
            .await;
    }
}

// in HW v0.2 pool usage would be 1.
// single task pool consume 864 bytes
// instance include usart without dma buffer consume 28 bytes
#[embassy_executor::task(pool_size = 1)]
pub async fn card_reader_device_spawn(instance: &'static CardReaderDevice) {
    instance.run().await
}

pub fn alert_module_status() {
    match SerialPayVarient::is_nda() {
        true => {
            defmt::info!("The module use a library for NDA devices.");
        }
        false => {
            defmt::warn!("The module use a example library. It may not work in real fields.");
        }
    }
}
