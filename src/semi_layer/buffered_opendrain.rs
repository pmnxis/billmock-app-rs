/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use core::cell::UnsafeCell;

use bit_field::BitField;
use embassy_stm32::gpio::{AnyPin, Output};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{with_timeout, Duration, Instant, Timer};

use super::timing::{DualPoleToggleTiming, ToggleTiming};

pub const HOST_SIDE_INTERFACE_CH_SIZE: usize = 2;
pub type OpenDrainRequestChannel =
    Channel<ThreadModeRawMutex, RawBufferedOpenDrainRequest, HOST_SIDE_INTERFACE_CH_SIZE>;

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

pub struct AltTickTockRequest {
    pub toggle_count: u8,
    pub timing: ToggleTiming,
}

pub enum BufferedOpenDrainRequest {
    /// Set low
    SetLow,
    /// Set high
    SetHigh,
    /// Set high and low with shared configuration
    TickTock(u8),
    /// Set high and low with alternative light configuration.
    /// Alternative light configuration is passing with limited paramter.
    /// toggle_count : u4(1..15)
    /// on/off_time  : u5(1..31) * 10 ms
    AltTickTock(AltTickTockRequest),
    /// Forever blink until not cancel
    ForeverBlink,
    /// Forever blink until not cancel with alternative light configuration.
    /// Alternative light configuration is passing with limited paramter.
    /// on/off_time  : u7(1..127) * 10 ms
    AltForeverBlink(ToggleTiming),
}

pub type RawBufferedOpenDrainRequest = u16;

const BF_15_14_OTHERS: u16 = 0b00;
const BF_15_14_ALT_TICKTOCK: u16 = 0b10;
const BF_15_14_ALT_FOREVER_BLINK: u16 = 0b11;

const BF_13_8_SET_LOW: u16 = 0b00_0000;
const BF_13_8_SET_HIGH: u16 = 0b00_0001;
const BF_13_8_TICKTOCK: u16 = 0b00_0010;
const BF_13_8_FOREVER_BLINK: u16 = 0b00_0011;

pub struct RawBufferedOpenDrainRequestTryIntoError {
    pub inner: RawBufferedOpenDrainRequest,
}

impl TryFrom<RawBufferedOpenDrainRequest> for BufferedOpenDrainRequest {
    type Error = RawBufferedOpenDrainRequestTryIntoError;

    fn try_from(value: RawBufferedOpenDrainRequest) -> Result<Self, Self::Error> {
        match value.get_bits(14..=15) {
            BF_15_14_OTHERS => match (value.get_bits(8..=13), value.get_bits(0..=7) as u8) {
                (BF_13_8_SET_LOW, 0) => Ok(Self::SetLow),
                (BF_13_8_SET_HIGH, 0) => Ok(Self::SetHigh),
                (BF_13_8_TICKTOCK, 0) => {
                    Err(RawBufferedOpenDrainRequestTryIntoError { inner: value })
                }
                (BF_13_8_TICKTOCK, x) => Ok(Self::TickTock(x)),
                (BF_13_8_FOREVER_BLINK, 0) => Ok(Self::ForeverBlink),
                _ => Err(RawBufferedOpenDrainRequestTryIntoError { inner: value }),
            },
            BF_15_14_ALT_TICKTOCK => match (
                value.get_bits(10..=13) as u8,
                value.get_bits(5..=9),
                value.get_bits(0..=4),
            ) {
                (0, _, _) | (_, 0, _) | (_, _, 0) => {
                    Err(RawBufferedOpenDrainRequestTryIntoError { inner: value })
                }
                (x, y, z) => Ok(Self::AltTickTock(AltTickTockRequest {
                    toggle_count: x,
                    timing: ToggleTiming {
                        high_ms: y * 10,
                        low_ms: z * 10,
                    },
                })),
            },
            BF_15_14_ALT_FOREVER_BLINK => match (value.get_bits(7..=13), value.get_bits(0..=6)) {
                (0, _) | (_, 0) => Err(RawBufferedOpenDrainRequestTryIntoError { inner: value }),
                (x, y) => Ok(Self::AltForeverBlink(ToggleTiming {
                    high_ms: x * 10,
                    low_ms: y * 10,
                })),
            },
            _ => Err(RawBufferedOpenDrainRequestTryIntoError { inner: value }),
        }
    }
}

impl From<&BufferedOpenDrainRequest> for RawBufferedOpenDrainRequest {
    fn from(value: &BufferedOpenDrainRequest) -> Self {
        match value {
            BufferedOpenDrainRequest::SetLow => BF_13_8_SET_LOW << 8,
            BufferedOpenDrainRequest::SetHigh => BF_13_8_SET_HIGH << 8,
            BufferedOpenDrainRequest::TickTock(x) => *x as u16 + (BF_13_8_TICKTOCK << 8),
            BufferedOpenDrainRequest::AltTickTock(AltTickTockRequest {
                toggle_count,
                timing: ToggleTiming { high_ms, low_ms },
            }) => {
                let mut ret = 0u16;
                *ret.set_bits(14..=15, BF_15_14_ALT_TICKTOCK)
                    .set_bits(10..=13, *toggle_count as u16)
                    .set_bits(5..=9, (*high_ms / 10).min(1 << 5))
                    .set_bits(0..=4, (*low_ms / 10).min(1 << 5))
            }
            BufferedOpenDrainRequest::ForeverBlink => BF_13_8_FOREVER_BLINK << 8,
            BufferedOpenDrainRequest::AltForeverBlink(ToggleTiming { high_ms, low_ms }) => {
                let mut ret = 0u16;
                *ret.set_bits(14..=15, BF_15_14_ALT_FOREVER_BLINK)
                    .set_bits(7..=13, (*high_ms / 10).min(1 << 7))
                    .set_bits(0..=6, (*low_ms / 10).min(1 << 7))
            }
        }
    }
}

impl From<BufferedOpenDrainRequest> for RawBufferedOpenDrainRequest {
    fn from(value: BufferedOpenDrainRequest) -> Self {
        match value {
            BufferedOpenDrainRequest::SetLow => BF_13_8_SET_LOW << 8,
            BufferedOpenDrainRequest::SetHigh => BF_13_8_SET_HIGH << 8,
            BufferedOpenDrainRequest::TickTock(x) => x as u16 + (BF_13_8_TICKTOCK << 8),
            BufferedOpenDrainRequest::AltTickTock(AltTickTockRequest {
                toggle_count,
                timing: ToggleTiming { high_ms, low_ms },
            }) => {
                let mut ret = 0u16;
                *ret.set_bits(14..=15, BF_15_14_ALT_TICKTOCK)
                    .set_bits(10..=13, toggle_count as u16)
                    .set_bits(5..=9, (high_ms / 10).min(1 << 5))
                    .set_bits(0..=4, (low_ms / 10).min(1 << 5))
            }
            BufferedOpenDrainRequest::ForeverBlink => BF_13_8_FOREVER_BLINK << 8,
            BufferedOpenDrainRequest::AltForeverBlink(ToggleTiming { high_ms, low_ms }) => {
                let mut ret = 0u16;
                *ret.set_bits(14..=15, BF_15_14_ALT_FOREVER_BLINK)
                    .set_bits(7..=13, (high_ms / 10).min(1 << 7))
                    .set_bits(0..=6, (low_ms / 10).min(1 << 7))
            }
        }
    }
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
    AltTickTock(NanoFsm, ToggleTiming),
    /// Forever blink until not cancel
    ForeverBlink(BlinkFsm),
    /// Forever blink until not cancel with alternative configuartion
    /// secondary shared configuration is to saving compute resource and RAM usage.
    AltForeverBlink(BlinkFsm, ToggleTiming),
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
            MicroHsm::AltTickTock(x, y) => Self::AltTickTock(AltTickTockRequest {
                toggle_count: x.toggle_count,
                timing: y,
            }),
            MicroHsm::ForeverBlink(_) => Self::ForeverBlink,
            MicroHsm::AltForeverBlink(_, y) => Self::AltForeverBlink(y),
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
            BufferedOpenDrainRequest::AltTickTock(x) => Self::AltTickTock(
                NanoFsm {
                    toggle_count: x.toggle_count,
                    state: true,
                    duration: timing.alt.high_ms,
                },
                x.timing,
            ),
            BufferedOpenDrainRequest::ForeverBlink => Self::ForeverBlink(BlinkFsm {
                state: true,
                duration: timing.shared.get().high_ms,
            }),
            BufferedOpenDrainRequest::AltForeverBlink(x) => Self::AltForeverBlink(
                BlinkFsm {
                    state: true,
                    duration: timing.alt.high_ms,
                },
                x,
            ),
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
            Self::AltTickTock(fsm, builtin_timing) => fsm
                .try_substract(*builtin_timing, elapsed)
                .map_or(Self::default(), |f| Self::AltTickTock(f, *builtin_timing)),
            Self::ForeverBlink(fsm) => {
                Self::ForeverBlink(fsm.substract(timing.shared.get(), elapsed))
            }
            Self::AltForeverBlink(fsm, builtin_timing) => {
                Self::AltForeverBlink(fsm.substract(*builtin_timing, elapsed), *builtin_timing)
            }
        }
    }

    pub fn next_sched_time(&self) -> Option<u16> {
        match self {
            Self::TickTock(fsm) | Self::AltTickTock(fsm, _) => Some(fsm.duration),
            Self::ForeverBlink(fsm) | Self::AltForeverBlink(fsm, _) => Some(fsm.duration),
            _ => None,
        }
    }

    pub fn expect_output_pin_state(&self) -> bool {
        match self {
            Self::SetLow => false,
            Self::SetHigh => true,
            Self::TickTock(fsm) | Self::AltTickTock(fsm, _) => fsm.expect_output_pin_state(),
            Self::ForeverBlink(fsm) | Self::AltForeverBlink(fsm, _) => {
                fsm.expect_output_pin_state()
            }
        }
    }

    pub fn is_busy(&self) -> bool {
        match self {
            Self::SetHigh | Self::SetLow => false,
            Self::TickTock(_) | Self::AltTickTock(_, _) => true,
            Self::ForeverBlink(_) | Self::AltForeverBlink(_, _) => false,
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
                        self.channel_hsm.receive(),
                    )
                    .await
                }
                (None, false) => Ok(self.channel_hsm.receive().await),
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

            match request.map_or(None, |x| {
                BufferedOpenDrainRequest::try_from(x).map_or_else(
                    |e| {
                        defmt::error!(
                            "RawBufferedOpenDrainRequestTryIntoError : 0x{:04X}",
                            e.inner
                        );
                        None
                    },
                    Some,
                )
            }) {
                Some(y) => {
                    hsm = (y, self.timing).into();
                }
                None => {
                    let elapsed = (Instant::now() - last).as_millis().min(u16::MAX.into()) as u16;
                    hsm = hsm.next(self.timing, elapsed);
                }
            }

            self.reflect_on_io(&hsm);
            last = Instant::now();
        }
    }

    pub async fn request(&self, request: BufferedOpenDrainRequest) {
        self.channel_hsm.send(request.into()).await
    }

    pub async fn try_request(&self, request: BufferedOpenDrainRequest) -> Result<(), ()> {
        self.channel_hsm
            .try_send(request.into())
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
    /// Not wait for being reflected but wait for sending queue.
    pub async fn tick_tock(&self, count: u8) {
        self.request(BufferedOpenDrainRequest::TickTock(count))
            .await
    }

    /// Simply order tick tock (high/low with alt duration configuration) on the opendrain module.
    /// Not wait for being reflected but wait for sending queue.
    pub async fn alt_tick_tock(&self, count: u8, high_ms: u16, low_ms: u16) {
        self.request(BufferedOpenDrainRequest::AltTickTock(AltTickTockRequest {
            toggle_count: count,
            timing: ToggleTiming { high_ms, low_ms },
        }))
        .await
    }

    /// Simply order tick tock (high/low with alt duration configuration) on the opendrain module.
    /// Not wait for being reflected but wait for sending queue.
    pub async fn alt_tick_tock_timing(&self, count: u8, timing: ToggleTiming) {
        self.request(BufferedOpenDrainRequest::AltTickTock(AltTickTockRequest {
            toggle_count: count,
            timing,
        }))
        .await
    }

    /// Simply order blink forever (high/low with shared duration configuration) on the opendrain module.
    /// Not wait for being reflected but wait for sending queue.
    pub async fn forever_blink(&self) {
        self.request(BufferedOpenDrainRequest::ForeverBlink).await
    }

    /// Simply order blink forever (high/low with alt duration configuration) on the opendrain module.
    /// Not wait for being reflected but wait for sending queue.
    pub async fn alt_forever_blink(&self, high_ms: u16, low_ms: u16) {
        self.request(BufferedOpenDrainRequest::AltForeverBlink(ToggleTiming {
            high_ms,
            low_ms,
        }))
        .await
    }

    /// Simply order blink forever (high/low with alt duration configuration) on the opendrain module.
    /// Not wait for being reflected but wait for sending queue.
    pub async fn alt_forever_blink_timing(&self, timing: ToggleTiming) {
        self.request(BufferedOpenDrainRequest::AltForeverBlink(timing))
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
