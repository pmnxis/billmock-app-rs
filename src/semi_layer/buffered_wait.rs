/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use core::cell::UnsafeCell;

use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::AnyPin;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Instant;

pub const MPSC_WAIT_INPUT_EVENT_CH_SIZE: usize = 32;

pub type RawInputPortKind = u8;

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
    pub port: RawInputPortKind,
    pub kind: TinyInputEventKind,
}

pub type InputEventChannel = Channel<ThreadModeRawMutex, InputEvent, MPSC_WAIT_INPUT_EVENT_CH_SIZE>;

pub type TinyInputEventKind = u8;
const TINY_LONG_PRESS_MAX: u8 = (0x1 << 7) - 1;

impl From<TinyInputEventKind> for InputEventKind {
    fn from(value: TinyInputEventKind) -> Self {
        if value == 0 {
            Self::Released
        } else {
            match value {
                0b1000_0000 => Self::Pressed,
                x => Self::LongPressed(x & 0b0111_1111),
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
pub struct BufferedWait {
    wait: UnsafeCell<ExtiInput<'static, AnyPin>>,
    port: RawInputPortKind,
    channel: &'static InputEventChannel,
}

#[allow(unused)]
impl BufferedWait {
    pub const fn new(
        wait: ExtiInput<'static, AnyPin>,
        port: RawInputPortKind,
        channel: &'static InputEventChannel,
    ) -> BufferedWait {
        Self {
            wait: UnsafeCell::new(wait),
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

    pub async fn run(&self) -> ! {
        // wait high for fit ot initial state.
        let wait = unsafe { &mut *self.wait.get() };
        wait.wait_for_high().await;

        loop {
            // detect low signal (active low)
            wait.wait_for_low().await;
            let entered_time = Instant::now();
            self.send(InputEventKind::Pressed).await;

            // detect high signal (active high)
            wait.wait_for_high().await;
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

// in HW v0.2 pool usage would be 6.
// PCB use 6 EXTI
// single task pool consume 88 bytes
#[embassy_executor::task(pool_size = 6)]
pub async fn buffered_wait_spawn(instance: &'static BufferedWait) {
    instance.run().await
}
