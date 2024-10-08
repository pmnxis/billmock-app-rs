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

impl defmt::Format for InputEventKind {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            InputEventKind::Released => defmt::write!(fmt, "Released"),
            InputEventKind::Pressed => defmt::write!(fmt, "Pressed"),
            InputEventKind::LongPressed(x) => defmt::write!(fmt, "LongPressed({})", x),
        }
    }
}

pub struct RawInputEvent {
    pub port: RawInputPortKind,
    pub event: RawInputEventKind,
}

pub type InputEventChannel =
    Channel<ThreadModeRawMutex, RawInputEvent, MPSC_WAIT_INPUT_EVENT_CH_SIZE>;

pub type RawInputEventKind = u8;
const TINY_LONG_PRESS_MAX: u8 = (0x1 << 7) - 1;

impl From<RawInputEventKind> for InputEventKind {
    fn from(value: RawInputEventKind) -> Self {
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

impl From<InputEventKind> for RawInputEventKind {
    fn from(value: InputEventKind) -> Self {
        match value {
            InputEventKind::Released => 0x00,
            InputEventKind::Pressed => 0x1 << 7,
            InputEventKind::LongPressed(x) => (0x1 << 7) | x.clamp(1, TINY_LONG_PRESS_MAX),
        }
    }
}

/// Internal PullUp + 4050 + OpenDrain outside (NMOS or ULN2803)
pub struct BufferedWait {
    wait: UnsafeCell<ExtiInput<'static, AnyPin>>,
    channel: &'static InputEventChannel,
    port: RawInputPortKind,
    #[cfg(debug_assertions)]
    debug_name: &'static str,
}

#[allow(unused)]
impl BufferedWait {
    pub const fn new(
        wait: ExtiInput<'static, AnyPin>,
        channel: &'static InputEventChannel,
        port: RawInputPortKind,
        debug_name: &'static str,
    ) -> BufferedWait {
        Self {
            wait: UnsafeCell::new(wait),
            channel,
            port,

            #[cfg(debug_assertions)]
            debug_name,
        }
    }

    async fn send(&self, event: InputEventKind) {
        self.channel
            .send(RawInputEvent {
                port: self.port,
                event: event.into(),
            })
            .await;
    }

    pub async fn run(&self) -> ! {
        // wait high for fit ot initial state.
        let wait = unsafe { &mut *self.wait.get() };
        wait.wait_for_high().await;

        #[cfg(debug_assertions)]
        defmt::println!("IN [{}  ] : High", self.debug_name);

        loop {
            // detect low signal (active low)
            wait.wait_for_low().await;
            let entered_time = Instant::now();

            #[cfg(debug_assertions)]
            defmt::println!("IN [{}  ] : Low", self.debug_name);

            self.send(InputEventKind::Pressed).await;

            // detect high signal (active high)
            wait.wait_for_high().await;
            let hold_time = Instant::now() - entered_time;

            #[cfg(debug_assertions)]
            defmt::println!(
                "IN [{}  ] : High, duration : {=u64:us}",
                self.debug_name,
                hold_time.as_micros()
            );

            match (hold_time.as_millis().min(TINY_LONG_PRESS_MAX as u64 * 10) / 10)
                as RawInputEventKind
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

// in HW v0.4 pool usage would be 6.
// single task pool consume 88 bytes
#[cfg(not(feature = "svc_button"))]
#[embassy_executor::task(pool_size = 6)]
pub async fn buffered_wait_spawn(instance: &'static BufferedWait) {
    instance.run().await
}

// in HW v0.5 pool usage would be 7. (+ SVC_Button)
// single task pool consume 88 bytes
#[cfg(feature = "svc_button")]
#[embassy_executor::task(pool_size = 7)]
pub async fn buffered_wait_spawn(instance: &'static BufferedWait) {
    instance.run().await
}
