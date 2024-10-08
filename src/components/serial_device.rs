/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use core::cell::UnsafeCell;

use billmock_plug_card::*;
use card_terminal_adapter::types::*;
use card_terminal_adapter::*;
use embassy_stm32::peripherals::USART2;
use embassy_stm32::peripherals::{DMA1_CH1, DMA1_CH2};
use embassy_stm32::usart::{RingBufferedUartRx, UartTx};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{with_timeout, Duration, Instant};

use crate::components::eeprom::{self, *};
use crate::const_str;

const CARD_READER_COMMAND_CHANNEL_SIZE_RX: usize = 8;
const CARD_READER_COMMAND_CHANNEL_SIZE_TX: usize = 16;
const WAIT_DURATION_RX: Duration = Duration::from_millis(200); // heuristic value
const WAIT_DURATION_TX: Duration = Duration::from_millis(3000); // heuristic value

pub const CARD_READER_RX_BUFFER_SIZE: usize = 128; // most of packet is 320~330 bytes
pub const CARD_READER_TX_BUFFER_SIZE: usize = 128; // most of pakcet is 6~12 bytes, but some uncommon command can be long

pub type CardReaderResponseChannel =
    Channel<ThreadModeRawMutex, CardTerminalRxCmd, CARD_READER_COMMAND_CHANNEL_SIZE_RX>;

pub type CardReaderRequestChannel =
    Channel<ThreadModeRawMutex, CardTerminalTxCmd, CARD_READER_COMMAND_CHANNEL_SIZE_TX>;

pub struct CardReaderDevice {
    // USART is complex to use generic
    tx: UnsafeCell<UartTx<'static, USART2, DMA1_CH2>>,
    rx: UnsafeCell<RingBufferedUartRx<'static, USART2, DMA1_CH1>>, // USART is complex to use generic
    pub recv_channel: CardReaderResponseChannel,
    pub req_channel: CardReaderRequestChannel,
}

type StackedRingbufferRxIndex = usize;

const ALT_TID: [u8; TID_LEN] = *b"  loading ";
async fn get_tid_alt(novella: &Novella) -> [u8; TID_LEN] {
    let tid = novella.lock_read(eeprom::select::TERMINAL_ID).await.normal;

    for a in tid {
        if !(b' '..=b'z').contains(&a) {
            return ALT_TID;
        }
    }

    tid
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

    pub async fn run(&self, novella: &'static Novella) {
        let plug = KiccEd785Plug {};
        let rx = unsafe { &mut *self.rx.get() };
        let tx = unsafe { &mut *self.tx.get() };
        let mut rx_buf = [0u8; CARD_READER_RX_BUFFER_SIZE];
        let mut tx_buf = [0u8; CARD_READER_TX_BUFFER_SIZE];
        let mut stacked: StackedRingbufferRxIndex = 0;
        let mut last_tx = Instant::now();

        loop {
            // TX not hang on IO wait
            if stacked == 0 {
                let now = Instant::now();

                // Card terminal has shallow buffer on receive
                // Billmock should consider it's processing time.
                if let Some(Ok(tx_cmd)) =
                    ((last_tx + WAIT_DURATION_TX) < now).then(|| self.req_channel.try_receive())
                {
                    defmt::info!("CardTerminalTxCmd : {}", tx_cmd);

                    let send_source = match tx_cmd {
                        CardTerminalTxCmd::Ack => plug.response_ack(&mut tx_buf),
                        CardTerminalTxCmd::Nack => plug.response_nack(&mut tx_buf),
                        CardTerminalTxCmd::ResponseDeviceInfo => plug.response_device_info(
                            &mut tx_buf,
                            &const_str::VERSION_STR,
                            const_str::get_serial_number(),
                        ),
                        CardTerminalTxCmd::PushCoinPaperAcceptorIncome(x) => {
                            last_tx = now;

                            plug.alert_coin_paper_acceptor_income(&mut tx_buf, x)
                        }
                        CardTerminalTxCmd::PushSaleSlotInfo => {
                            let slot_info =
                                novella.lock_read(eeprom::select::CARD_PORT_BACKUP).await;

                            plug.push_sale_slot_info(&mut tx_buf, &slot_info)
                        }
                        CardTerminalTxCmd::PushSaleSlotInfoPartialInhibit(x) => {
                            // Unfortunately, this feature is still unstable due to sequence logic issues
                            // in the real environment. Therefore, we do not recommend using it at this time.
                            let mut slot_info =
                                novella.lock_read(eeprom::select::CARD_PORT_BACKUP).await;
                            slot_info.set_inhibit(x);

                            let ret =
                                plug.push_sale_slot_info_partial_inhibit(&mut tx_buf, &slot_info);

                            novella
                                .lock_write(eeprom::select::CARD_PORT_BACKUP, slot_info)
                                .await;

                            ret
                        }
                        CardTerminalTxCmd::SetTransactionAvailability(is_avail) => {
                            plug.push_transaction_availability(&mut tx_buf, is_avail)
                        }
                        CardTerminalTxCmd::RequestSaleSlotInfo => {
                            plug.request_sale_slot_info(&mut tx_buf)
                        }
                        CardTerminalTxCmd::RequestTerminalInfo => {
                            plug.request_terminal_info(&mut tx_buf)
                        }
                        CardTerminalTxCmd::DisplayRom => {
                            let p1_card = novella.lock_read(eeprom::select::P1_CARD_CNT).await;
                            let p2_card = novella.lock_read(eeprom::select::P2_CARD_CNT).await;
                            let p1_coin = novella.lock_read(eeprom::select::P1_COIN_CNT).await;
                            let p2_coin = novella.lock_read(eeprom::select::P2_COIN_CNT).await;
                            let tid = get_tid_alt(novella).await;

                            plug.display_rom(
                                &mut tx_buf,
                                &const_str::COMMIT_SHORT,
                                &tid,
                                p1_card,
                                p2_card,
                                p1_coin,
                                p2_coin,
                            )
                        }
                        CardTerminalTxCmd::DisplayHwInfo => {
                            let hw_boot_cnt = novella.lock_read(eeprom::select::HW_BOOT_CNT).await;
                            let tid = get_tid_alt(novella).await;
                            let uptime_minutes =
                                (novella.get_uptime().as_secs() / 60).min(u32::MAX as u64) as u32;

                            plug.display_hw_info(
                                &mut tx_buf,
                                &const_str::VERSION_STR,
                                const_str::get_serial_number(),
                                &tid,
                                hw_boot_cnt,
                                uptime_minutes,
                            )
                        }
                        CardTerminalTxCmd::DisplayWarning(x) => {
                            plug.display_warning(&mut tx_buf, x)
                        }
                    };

                    defmt::debug!("Tx Gen Buf : {:#X}", &send_source);

                    // send generated packet though uart dma
                    if let Err(e_dma) = tx.write(send_source).await {
                        defmt::error!("USART TX error : {:?}", e_dma);
                    }
                }
            }

            // RX work
            match with_timeout(WAIT_DURATION_RX, rx.read(&mut rx_buf[stacked..])).await {
                Ok(Ok(rx_len)) => {
                    // for debug
                    let re_len = stacked + rx_len;
                    let rx_source = &rx_buf[..re_len];
                    // defmt::debug!("UART READ {}: {:02X}", rx_len, &rx_source);
                    // end of debug

                    match plug.pre_parse_common(rx_source) {
                        Ok(rx_cmd) => {
                            let final_rx_cmd = match rx_cmd {
                                CardTerminalRxCmd::ResponseSaleSlotInfo => {
                                    let result = plug.post_parse_response_sale_slot_info(rx_source);

                                    if let Ok(x) = result {
                                        // todo! - considier SlotProperty::TemporaryDisabled
                                        // Unfortunately, this feature is still unstable due to sequence logic
                                        // issues in the real environment.
                                        // Therefore, we do not recommend using it at this time.

                                        novella
                                            .lock_write(eeprom::select::CARD_PORT_BACKUP, x)
                                            .await;

                                        Ok(rx_cmd)
                                    } else if let Err(e) = result {
                                        defmt::error!("CardTerminal Parse Error : {}", e);
                                        Err(())
                                    } else {
                                        Err(())
                                    }
                                }
                                CardTerminalRxCmd::ResponseTerminalInfo(_, _) => {
                                    let prev_tid =
                                        novella.lock_read(eeprom::select::TERMINAL_ID).await;

                                    let result = plug
                                        .post_parse_response_terminal_info(rx_source, &prev_tid);
                                    match result {
                                        Ok((ret, tid)) => match ret {
                                            CardTerminalRxCmd::ResponseTerminalInfo(
                                                TidStatus::Changed,
                                                _,
                                            ) => {
                                                defmt::info!(
                                                    "tid : {=[u8]:a} -> {=[u8]:a}",
                                                    prev_tid.normal,
                                                    tid.normal
                                                );

                                                novella
                                                    .lock_write(eeprom::select::TERMINAL_ID, tid)
                                                    .await;

                                                Ok(ret)
                                            }
                                            CardTerminalRxCmd::ResponseTerminalInfo(
                                                TidStatus::Unchanged,
                                                _,
                                            ) => {
                                                defmt::info!(
                                                    "tid : {=[u8]:a} (Unchanged)",
                                                    tid.normal
                                                );

                                                Ok(ret)
                                            }
                                            _ => {
                                                defmt::error!(
                                                    "Unexpected ResponseTerminalInfo error pass"
                                                );
                                                Err(())
                                            }
                                        },
                                        Err(e) => {
                                            defmt::error!("ResponseTerminalInfo error : {:?}", e);
                                            Err(())
                                        }
                                    }
                                }
                                x => Ok(x),
                            };

                            // finally, only parse when Ok(rx_cmd)
                            match final_rx_cmd {
                                Ok(rx_cmd) => {
                                    defmt::info!(
                                        "{:#X}\nCardTerminalRxCmd : {:?}, restack : {}",
                                        rx_source,
                                        rx_cmd,
                                        stacked != 0,
                                    );
                                    stacked = 0;
                                    self.recv_channel.send(rx_cmd).await;
                                }
                                Err(_e) => {}
                            }
                        }
                        Err(e) => {
                            defmt::error!("CardTerminal Parse Error : {}", e);
                            match e {
                                CardTerminalError::BadLength | CardTerminalError::InvalidFrame => {
                                    stacked += rx_len;
                                }
                                _ => {
                                    defmt::debug!("Rx Buf : {:#X}", rx_source);
                                }
                            }
                        }
                    }
                }
                Err(_timeout_e) => {
                    stacked = 0;
                }
                Ok(Err(e)) => {
                    stacked = 0;
                    defmt::error!("USART error : {:?}", e);
                }
            }
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub async fn send(&self, request: CardTerminalTxCmd) {
        self.req_channel.send(request).await;
    }

    pub async fn send_ack(&self) {
        self.req_channel.send(CardTerminalTxCmd::Ack).await;
    }

    pub async fn send_nack(&self) {
        self.req_channel.send(CardTerminalTxCmd::Nack).await;
    }

    pub async fn send_inhibit(&self, inhibit: RawPlayersInhibit) {
        self.req_channel
            .send(CardTerminalTxCmd::PushSaleSlotInfoPartialInhibit(inhibit))
            .await;
    }

    pub async fn send_transaction_availability(&self, is_avail: bool) {
        self.req_channel
            .send(CardTerminalTxCmd::SetTransactionAvailability(is_avail))
            .await;
    }
}

// in HW v0.2 pool usage would be 1.
// single task pool consume 864 bytes
// instance include usart without dma buffer consume 28 bytes
#[embassy_executor::task(pool_size = 1)]
pub async fn card_reader_device_spawn(
    instance: &'static CardReaderDevice,
    novella: &'static Novella,
) {
    instance.run(novella).await;
}

pub fn alert_module_status() {
    match KiccEd785Plug::is_nda() {
        true => {
            defmt::info!("The module use a library for NDA devices.");
        }
        false => {
            defmt::warn!("The module use a example library. It may not work in real fields.");
        }
    }
}
