/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use embedded_hal::digital::InputPin;
use {defmt_rtt as _, panic_probe as _};

use crate::types::{AppMode, PriceReflection, TimingOverride};

/*
 * embassy_stm32 Alert
 * You should enable feature `unstable-traits`
 */

pub struct DipSwitch<
    AnyPin0: InputPin,
    AnyPin1: InputPin,
    AnyPin2: InputPin,
    AnyPin3: InputPin,
    AnyPin4: InputPin,
    AnyPin5: InputPin,
> {
    gpios: (AnyPin0, AnyPin1, AnyPin2, AnyPin3, AnyPin4, AnyPin5),
}

#[allow(dead_code)]
impl<
        AnyPin0: InputPin,
        AnyPin1: InputPin,
        AnyPin2: InputPin,
        AnyPin3: InputPin,
        AnyPin4: InputPin,
        AnyPin5: InputPin,
    > DipSwitch<AnyPin0, AnyPin1, AnyPin2, AnyPin3, AnyPin4, AnyPin5>
{
    pub const fn new(
        in_price0: AnyPin0,
        in_price1: AnyPin1,
        in_timing0: AnyPin2,
        in_timing1: AnyPin3,
        in_mode0: AnyPin4,
        in_mode1: AnyPin5,
    ) -> Self {
        Self {
            gpios: (
                in_price0, in_price1, in_timing0, in_timing1, in_mode0, in_mode1,
            ),
        }
    }

    pub fn read(&self) -> (PriceReflection, TimingOverride, AppMode) {
        (
            PriceReflection::try_from(
                self.gpios.0.is_high().unwrap() as u8 + self.gpios.1.is_high().unwrap() as u8 * 2,
            )
            .unwrap(),
            TimingOverride::try_from(
                self.gpios.2.is_high().unwrap() as u8 + self.gpios.3.is_high().unwrap() as u8 * 2,
            )
            .unwrap(),
            AppMode::try_from(
                self.gpios.4.is_high().unwrap() as u8 + self.gpios.5.is_high().unwrap() as u8 * 2,
            )
            .unwrap(),
        )
    }
}
