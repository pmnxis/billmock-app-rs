/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use defmt::export::panic;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;
use heapless::Deque;
use {defmt_rtt as _, panic_probe as _};

pub struct VirtualStart<IN_SWITCH: Wait, OUT_LED: OutputPin> {
    // logic related
    /// Current duty period or is enabled
    current_duty: Option<BlinkDuty>,
    // hardware related
    /// Button input (floating input)
    in_switch: IN_SWITCH,
    /// Button indicator (should be push-pull)
    out_led: OUT_LED,
}

#[derive(Clone, Copy)]
pub struct BlinkDuty {
    on_ms: u16,
    off_ms: u16,
}

impl<IN_SWITCH: Wait, OUT_LED: OutputPin> VirtualStart<IN_SWITCH, OUT_LED> {
    pub fn new(in_switch: IN_SWITCH, mut out_led: OUT_LED) -> VirtualStart<IN_SWITCH, OUT_LED> {
        out_led.set_low().ok();

        VirtualStart {
            current_duty: None,
            in_switch,
            out_led,
        }
    }

    pub fn blink_on(mut self, duty: Option<BlinkDuty>) {
        self.current_duty = Some(match duty {
            None => BlinkDuty {
                on_ms: 100,
                off_ms: 0,
            },
            Some(x) => x,
        });
    }

    pub fn blink_off(mut self) {
        self.current_duty = None;
    }

    pub async fn run(mut self) -> ! {
        loop {
            match self.current_duty {
                Some(x) => {
                    // RwLock or Mutex required
                    if x.off_ms != 0 {
                        self.out_led.set_high().ok();
                        Timer::after(Duration::from_millis(x.on_ms.into())).await;

                        self.out_led.set_low().ok();
                        Timer::after(Duration::from_millis(x.off_ms.into())).await;
                    } else {
                        self.out_led.set_high().ok();
                        Timer::after(Duration::from_millis(100)).await;
                    }
                }
                None => {
                    self.out_led.set_low().ok();
                    Timer::after(Duration::from_millis(100)).await;
                }
            }
        }
    }
}
