/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use core::cell::UnsafeCell;

use embassy_stm32::gpio::{AnyPin, Output};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{with_timeout, Duration, Instant, Timer};

use super::timing::{DualPoleToggleTiming, ToggleTiming};

pub const HOST_SIDE_INTERFACE_CH_SIZE: usize = 2;
pub type OpenDrainRequestChannel =
    Channel<ThreadModeRawMutex, BufferedOpenDrainRequest, HOST_SIDE_INTERFACE_CH_SIZE>;

struct NanoFsm {
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
    fn try_substract(&self, timing: ToggleTiming, elapsed: u16) -> Result<Self, ()> {
        match self.duration > elapsed {
            true => Ok(NanoFsm {
                toggle_count: self.toggle_count,
                state: self.state,
                duration: self.duration - elapsed,
            }),
            false => match (self.toggle_count, self.state) {
                (0, _) | (1, false) => Err(()),
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
    fn substract(&self, timing: ToggleTiming, elapsed: u16) -> Self {
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

pub enum BufferedOpenDrainRequest {
    /// Set low
    SetLow,
    /// Set high
    SetHigh,
    /// Set high and low with shared configuration
    TickTock(u8),
    /// Set high and low with alternative means secondary shared configuration.
    /// secondary shared configuration is to saving compute resource and RAM usage.
    AltTickTock(u8),
    /// Forever blink until not cancel
    ForeverBlink,
    /// Forever blink until not cancel with alternative configuartion
    /// secondary shared configuration is to saving compute resource and RAM usage.
    AltForeverBlink,
}

enum MicroHsm {
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

impl From<MicroHsm> for BufferedOpenDrainRequest {
    fn from(value: MicroHsm) -> Self {
        match value {
            MicroHsm::SetLow => Self::SetLow,
            MicroHsm::SetHigh => Self::SetHigh,
            MicroHsm::TickTock(x) => Self::TickTock(x.toggle_count),
            MicroHsm::AltTickTock(x) => Self::AltTickTock(x.toggle_count),
            MicroHsm::ForeverBlink(_) => Self::ForeverBlink,
            MicroHsm::AltForeverBlink(_) => Self::AltForeverBlink,
        }
    }
}

impl From<(BufferedOpenDrainRequest, &'static DualPoleToggleTiming)> for MicroHsm {
    fn from((req, timing): (BufferedOpenDrainRequest, &'static DualPoleToggleTiming)) -> Self {
        match req {
            BufferedOpenDrainRequest::SetLow => Self::SetLow,
            BufferedOpenDrainRequest::SetHigh => Self::SetHigh,
            BufferedOpenDrainRequest::TickTock(x) => Self::TickTock(NanoFsm {
                toggle_count: x,
                state: true,
                duration: timing.shared.get().high_ms,
            }),
            BufferedOpenDrainRequest::AltTickTock(x) => Self::AltTickTock(NanoFsm {
                toggle_count: x,
                state: true,
                duration: timing.alt.high_ms,
            }),
            BufferedOpenDrainRequest::ForeverBlink => Self::ForeverBlink(BlinkFsm {
                state: true,
                duration: timing.shared.get().high_ms,
            }),
            BufferedOpenDrainRequest::AltForeverBlink => Self::AltForeverBlink(BlinkFsm {
                state: true,
                duration: timing.alt.high_ms,
            }),
        }
    }
}

impl MicroHsm {
    pub fn next(&self, timing: &'static DualPoleToggleTiming, elapsed: u16) -> Self {
        match self {
            Self::SetLow => Self::SetLow,
            Self::SetHigh => Self::SetHigh,
            Self::TickTock(fsm) => fsm
                .try_substract(timing.shared.get(), elapsed)
                .map_or(Self::default(), Self::TickTock),
            Self::AltTickTock(fsm) => fsm
                .try_substract(timing.alt, elapsed)
                .map_or(Self::default(), Self::AltTickTock),
            Self::ForeverBlink(fsm) => {
                Self::ForeverBlink(fsm.substract(timing.shared.get(), elapsed))
            }
            Self::AltForeverBlink(fsm) => Self::AltForeverBlink(fsm.substract(timing.alt, elapsed)),
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
    io: UnsafeCell<Output<'static, AnyPin>>,
    timing: &'static DualPoleToggleTiming,
    channel_hsm: OpenDrainRequestChannel,
}

#[allow(unused)]
impl BufferedOpenDrain {
    fn reflect_on_io(&self, hsm: &MicroHsm) {
        let io = unsafe { &mut *self.io.get() };

        io.set_level(hsm.expect_output_pin_state().into());
    }

    pub const fn new(
        out_pin: Output<'static, AnyPin>,
        timing: &'static DualPoleToggleTiming,
    ) -> Self {
        Self {
            io: UnsafeCell::new(out_pin),
            timing,
            channel_hsm: Channel::new(),
        }
    }

    async fn run(&self) {
        let mut hsm = MicroHsm::default();
        self.reflect_on_io(&hsm);

        let mut last = Instant::now();

        loop {
            let request = match (hsm.next_sched_time(), hsm.is_busy()) {
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
                    self.reflect_on_io(&hsm);

                    let elapsed = (Instant::now() - last).as_millis().min(u16::MAX.into()) as u16;

                    hsm = hsm.next(self.timing, elapsed);
                    self.reflect_on_io(&hsm);

                    continue;
                }
            };

            match request {
                Ok(received) => {
                    // replace slot
                    hsm = (received, self.timing).into();
                }
                Err(_) => {
                    let elapsed = (Instant::now() - last).as_millis().min(u16::MAX.into()) as u16;
                    hsm = hsm.next(self.timing, elapsed);
                }
            }

            self.reflect_on_io(&hsm);
            last = Instant::now();
        }
    }

    pub async fn request(&self, request: BufferedOpenDrainRequest) {
        self.channel_hsm.send(request).await
    }

    pub async fn try_request(&self, request: BufferedOpenDrainRequest) -> Result<(), ()> {
        self.channel_hsm
            .try_send(request)
            .map_or(Err(()), |_| Ok(()))
    }

    /// Simply order set high on the opendrain module, but doesn't wait for being reflected.
    pub async fn set_high(&self) {
        self.request(BufferedOpenDrainRequest::SetHigh).await
    }

    /// Simply order set low on the opendrain module, but doesn't wait for being reflected.
    pub async fn set_low(&self) {
        self.request(BufferedOpenDrainRequest::SetLow).await
    }

    /// Simply order set high or low opendrain module by boolean, but doesn't wait for being reflected.
    pub async fn set_level(&self, state: bool) {
        self.request(match state {
            true => BufferedOpenDrainRequest::SetHigh,
            false => BufferedOpenDrainRequest::SetLow,
        })
        .await
    }

    /// Simply order tick tock (high/low with shared duration configuration) on the opendrain module.
    /// Not wait for being reflected and wait for sending queue.
    pub async fn tick_tock(&self, count: u8) {
        self.request(BufferedOpenDrainRequest::TickTock(count))
            .await
    }

    /// Simply order tick tock (high/low with alt duration configuration) on the opendrain module.
    /// Not wait for being reflected and wait for sending queue.
    pub async fn alt_tick_tock(&self, count: u8) {
        self.request(BufferedOpenDrainRequest::AltTickTock(count))
            .await
    }

    /// Simply order blink forever (high/low with shared duration configuration) on the opendrain module.
    /// Not wait for being reflected and wait for sending queue.
    pub async fn forever_blink(&self) {
        self.request(BufferedOpenDrainRequest::ForeverBlink).await
    }

    /// Simply order blink forever (high/low with alt duration configuration) on the opendrain module.
    /// Not wait for being reflected and wait for sending queue.
    pub async fn alt_forever_blink(&self) {
        self.request(BufferedOpenDrainRequest::AltForeverBlink)
            .await
    }
}

// in HW v0.2 pool usage would be 13.
// PCB has 13 N-MOS open-drain.
// single task pool consume 112 bytes
#[embassy_executor::task(pool_size = 13)]
pub async fn buffered_opendrain_spawn(instance: &'static BufferedOpenDrain) {
    instance.run().await
}
