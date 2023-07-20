/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use embassy_stm32::gpio::{AnyPin, Output};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{with_timeout, Duration, Instant, Timer};

use super::timing::{DualPoleToggleTiming, ToggleTiming};

pub const HOST_SIDE_INTERFACE_CH_SIZE: usize = 2;
pub type OpenDrainRequestChannel =
    Channel<ThreadModeRawMutex, MicroHsm, HOST_SIDE_INTERFACE_CH_SIZE>;

pub struct NanoFsm {
    /// Given or left toggle number
    /// Mean number of cycle of `High` and `Low` combination
    toggle_count: u8,
    /// Current state of output
    /// start from true and next is false, after than count down toggle_count than turn true again
    state: bool,
    /// The time left for next event
    /// This value would be used internally
    duration: u16,
}

impl NanoFsm {
    /// Try substract `NanoFsm`
    /// Result with Error is toggle count, state, duration reached or underflowed zero
    /// Ok(Some) means there's changes
    fn try_substract(&self, timing: &ToggleTiming, elapsed: u16) -> Result<Self, ()> {
        match self.duration > elapsed {
            true => Ok(NanoFsm {
                toggle_count: self.toggle_count,
                state: self.state,
                duration: self.duration - elapsed,
            }),
            false => match (self.toggle_count, self.state) {
                (0, false) => Err(()),
                (toggle_count, true) => Ok(NanoFsm {
                    toggle_count,
                    state: false,
                    duration: timing.low_ms,
                }),
                (toggle_count, false) => Ok(NanoFsm {
                    toggle_count: toggle_count - 1,
                    state: true,
                    duration: timing.high_ms,
                }),
            },
        }
    }

    fn expect_output_pin_state(&self) -> bool {
        self.state
    }
}

/// FSM for ForeverBlink
pub struct BlinkFsm {
    /// Current state of output
    /// start from true and next is false, after than count down toggle_count than turn true again
    state: bool,
    /// The time left for next event
    /// This value would be used internally
    duration: u16,
}

impl BlinkFsm {
    /// Substract `BlinkFsm`
    fn substract(&self, timing: &ToggleTiming, elapsed: u16) -> Self {
        match self.duration > elapsed {
            true => BlinkFsm {
                state: self.state,
                duration: self.duration - elapsed,
            },
            false => match self.state {
                true => BlinkFsm {
                    state: false,
                    duration: timing.low_ms,
                },
                false => BlinkFsm {
                    state: true,
                    duration: timing.high_ms,
                },
            },
        }
    }

    fn expect_output_pin_state(&self) -> bool {
        self.state
    }
}

pub enum MicroHsm {
    /// Set low
    SetLow,
    /// Set high
    SetHigh,
    /// Set high and low with shared configuration
    TickTock(NanoFsm),
    /// Set high and low with alternative means secondary shared configuration.
    /// secondary shared configuration is to saving compute resource and RAM usage.
    AltTickTock(NanoFsm),
    /// Forever blink until not cancel
    ForeverBlink(BlinkFsm),
    /// Forever blink until not cancel with alternative configuartion
    /// secondary shared configuration is to saving compute resource and RAM usage.
    AltForeverBlink(BlinkFsm),
}

impl MicroHsm {
    /// Default init is `SetLow`
    pub const fn default() -> Self {
        Self::SetLow
    }
}

impl MicroHsm {
    pub fn next(&self, timing: &'static DualPoleToggleTiming, elapsed: u16) -> Self {
        match self {
            Self::SetLow => Self::SetLow,
            Self::SetHigh => Self::SetHigh,
            Self::TickTock(fsm) => fsm
                .try_substract(&timing.shared.get(), elapsed)
                .map_or(Self::default(), |f| Self::TickTock(f)),
            Self::AltTickTock(fsm) => fsm
                .try_substract(&timing.alt, elapsed)
                .map_or(Self::default(), |f: NanoFsm| Self::AltTickTock(f)),
            Self::ForeverBlink(fsm) => {
                Self::ForeverBlink(fsm.substract(&timing.shared.get(), elapsed))
            }
            Self::AltForeverBlink(fsm) => {
                Self::AltForeverBlink(fsm.substract(&timing.alt, elapsed))
            }
        }
    }

    pub fn next_sched_time(&self) -> Option<u16> {
        match self {
            Self::TickTock(fsm) | Self::AltTickTock(fsm) => Some(fsm.duration),
            Self::ForeverBlink(fsm) | Self::AltForeverBlink(fsm) => Some(fsm.duration),
            _ => None,
        }
    }

    pub fn expect_output_pin_state(&self) -> bool {
        match self {
            Self::SetLow => false,
            Self::SetHigh => true,
            Self::TickTock(fsm) | Self::AltTickTock(fsm) => fsm.expect_output_pin_state(),
            Self::ForeverBlink(fsm) | Self::AltForeverBlink(fsm) => fsm.expect_output_pin_state(),
        }
    }

    pub fn is_busy(&self) -> bool {
        match self {
            Self::SetHigh | Self::SetLow => false,
            Self::TickTock(_) | Self::AltTickTock(_) => true,
            Self::ForeverBlink(_) | Self::AltForeverBlink(_) => false,
        }
    }
}

pub struct BufferedOpenDrain {
    io: Output<'static, AnyPin>,
    timing: &'static DualPoleToggleTiming,
    channel_hsm: OpenDrainRequestChannel,
    hsm: MicroHsm,
}

impl BufferedOpenDrain {
    pub fn reflect_on_io(&mut self) {
        match self.hsm.expect_output_pin_state() {
            false => self.io.set_low(),
            true => self.io.set_high(),
        };
    }

    pub fn next_sched_time(&self) -> Option<u16> {
        self.hsm.next_sched_time()
    }

    pub const fn new(
        mut out_pin: Output<'static, AnyPin>,
        timing: &'static DualPoleToggleTiming,
    ) -> Self {
        Self {
            io: out_pin,
            timing,
            channel_hsm: Channel::new(),
            hsm: MicroHsm::default(),
        }
    }

    pub async fn run(&mut self) {
        self.reflect_on_io();
        let mut last = Instant::now();

        loop {
            let request = match (self.next_sched_time(), self.hsm.is_busy()) {
                (Some(wait_ms), false) => {
                    with_timeout(
                        Duration::from_millis(wait_ms.into()),
                        self.channel_hsm.recv(),
                    )
                    .await
                }
                (None, false) => Ok(self.channel_hsm.recv().await),
                // Not allowed in busy, that means not-interruptable
                (Some(wait_ms), true) => {
                    Timer::after(Duration::from_millis(wait_ms.into())).await;
                    Err(embassy_time::TimeoutError)
                }
                (None, true) => {
                    // this would be happend rarely
                    self.reflect_on_io();

                    let elapsed = (Instant::now() - last).as_millis().min(u16::MAX.into()) as u16;

                    self.hsm = self.hsm.next(self.timing, elapsed);
                    self.reflect_on_io();

                    continue;
                }
            };

            match request {
                Ok(received) => {
                    // replace slot
                    self.hsm = received;
                }
                Err(_) => {
                    let elapsed = (Instant::now() - last).as_millis().min(u16::MAX.into()) as u16;
                    self.hsm = self.hsm.next(self.timing, elapsed);
                }
            }

            self.reflect_on_io();
            last = Instant::now();
        }
    }
}

// in HW v0.2 pool usage would be 13.
// PCB has 13 N-MOS open-drain.
#[embassy_executor::task(pool_size = 16)]
pub async fn buffered_opendrain_spawn(instance: &'static mut BufferedOpenDrain) {
    instance.run().await
}
