/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embedded_hal_async::digital::Wait;

pub const MPSC_WAIT_INPUT_EVENT_CH_SIZE: usize = 32;

// InputPortKind would be replace to
// pub type InputPortKind = u8,
// external enum with #[repr(u8)] will be used in other code space
#[derive(Debug, Clone, Copy)]
pub enum InputPortKind {
    Start1P,
    Start2P,
    Vend,
    Jam,
    Inhibit1,
    Inhibit2,
}

// 8bit-sized enum
#[derive(Debug, Clone, Copy)]
pub enum InputEventKind {
    /// Released signal (active High)
    Released,
    /// Pressed signal (active Low)
    Pressed,
    /// Long press signal with time
    /// Internal time max value is 0x7F (127), the value * 10 is pressed time in Msecs
    LongPressed(u8),
}

pub struct InputEvent {
    port: InputPortKind,
    kind: TinyInputEventKind,
}

pub type TinyInputEventKind = u8;
const TINY_LONG_PRESS_MAX: u8 = (0x1 << 7) - 1;

impl From<TinyInputEventKind> for InputEventKind {
    fn from(value: TinyInputEventKind) -> Self {
        if (value & (0x1 << 7) != 0) {
            return Self::Released;
        } else {
            match value & 0b01111111 {
                0 => Self::Pressed,
                x => Self::LongPressed(x),
            }
        }
    }
}

impl From<InputEventKind> for TinyInputEventKind {
    fn from(value: InputEventKind) -> Self {
        match value {
            InputEventKind::Released => 0x00,
            InputEventKind::Pressed => 0x1 << 7,
            InputEventKind::LongPressed(x) => (0x1 << 7) | x.max(1).min(TINY_LONG_PRESS_MAX),
        }
    }
}

/// Internal PullUp + 4050 + OpenDrain outside (NMOS or ULN2803)
pub struct BufferedWait<WaitIo: Wait> {
    wait: WaitIo,
    port: InputPortKind,
    channel: &'static Channel<ThreadModeRawMutex, InputEvent, MPSC_WAIT_INPUT_EVENT_CH_SIZE>,
}

impl<WaitIo: Wait> BufferedWait<WaitIo> {
    fn new(
        mut wait: WaitIo,
        port: InputPortKind,
        channel: &'static Channel<ThreadModeRawMutex, InputEvent, MPSC_WAIT_INPUT_EVENT_CH_SIZE>,
    ) -> BufferedWait<WaitIo> {
        BufferedWait {
            wait,
            port,
            channel,
        }
    }

    async fn send(&self, event: InputEventKind) {
        self.channel
            .send(InputEvent {
                port: self.port,
                kind: event.into(),
            })
            .await;
    }

    pub async fn run(&mut self) -> ! {
        // wait high for fit ot initial state.
        self.wait.wait_for_high().await;

        loop {
            // detect low signal (active low)
            self.wait.wait_for_low().await;
            let entered_time = Instant::now();
            self.send(InputEventKind::Pressed).await;

            // detect high signal (active high)
            self.wait.wait_for_high().await;
            match ((Instant::now() - entered_time)
                .as_millis()
                .min(TINY_LONG_PRESS_MAX as u64 * 10)
                / 10) as TinyInputEventKind
            {
                0 => { /* too short time pressed */ }
                x => {
                    self.send(InputEventKind::LongPressed(x)).await;
                }
            }
            self.send(InputEventKind::Released).await;
        }
    }
}
