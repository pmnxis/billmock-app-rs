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

use super::buffered_wait::{InputEventChannel, InputEventKind, RawInputEvent};

// #[allow(unused)]
pub struct BufferedWaitReceiver {
    pub channel: InputEventChannel,
    bit_cache: Mutex<ThreadModeRawMutex, u16>,
}

impl BufferedWaitReceiver {
    pub const fn new() -> Self {
        Self {
            channel: Channel::new(),
            bit_cache: Mutex::new(0),
        }
    }

    pub fn try_recv(&'static self) -> Result<RawInputEvent, TryRecvError> {
        let received = self.channel.try_recv()?;
        let event = InputEventKind::from(received.event);

        let port_num = received.port as usize;
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
    pub async fn recv(&'static self) -> RawInputEvent {
        let received = self.channel.recv().await;
        let event = InputEventKind::from(received.event);

        let port_num = received.port as usize;
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
        self.bit_cache.lock(|x| *x)
    }

    #[allow(dead_code)]
    pub fn get_cache_optional(&self, other: u16) -> Option<u16> {
        let me = self.get_cache();
        if me == other {
            None
        } else {
            Some(me)
        }
    }
}
