/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use bit_field::BitField;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::TryRecvError;
use static_cell::make_static;

use super::buffered_wait::{InputEventChannel, InputEventKind, RawInputEvent};

// #[allow(unused)]
pub struct BufferedWaitReceiver {
    pub channel: &'static InputEventChannel,
    bit_cache: Mutex<ThreadModeRawMutex, u16>,
}

impl BufferedWaitReceiver {
    pub fn new() -> Self {
        Self {
            channel: make_static!(Channel::new()),
            bit_cache: Mutex::new(0),
        }
    }

    pub fn try_recv(&self) -> Result<RawInputEvent, TryRecvError> {
        let received = self.channel.try_recv()?;
        let event = InputEventKind::from(received.event);

        let port_num = u8::from(received.port) as usize;
        match event {
            InputEventKind::Pressed => {
                self.bit_cache.lock(|x| {
                    let mut x = *x;
                    x.set_bit(port_num, true);
                });
            }
            InputEventKind::Released => {
                self.bit_cache.lock(|x| {
                    let mut x = *x;
                    x.set_bit(port_num, false);
                });
            }
            _ => {}
        };

        Ok(received)
    }

    #[allow(dead_code)]
    pub async fn recv(&self) -> RawInputEvent {
        let received = self.channel.recv().await;
        let event = InputEventKind::from(received.event);

        let port_num = u8::from(received.port) as usize;
        match event {
            InputEventKind::Pressed => {
                self.bit_cache.lock(|x| {
                    let mut x = *x;
                    x.set_bit(port_num, true);
                });
            }
            InputEventKind::Released => {
                self.bit_cache.lock(|x| {
                    let mut x = *x;
                    x.set_bit(port_num, false);
                });
            }
            _ => {}
        };

        received
    }

    pub fn get_cache(&self) -> u16 {
        self.bit_cache.lock(|x| x.clone())
    }
}
