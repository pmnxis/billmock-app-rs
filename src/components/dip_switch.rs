/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use embassy_stm32::gpio::{AnyPin, Input};
use {defmt_rtt as _, panic_probe as _};

use crate::types::{AppMode, PriceReflection, TimingOverride};

/*
 * embassy_stm32 Alert
 * You should enable feature `unstable-traits`
 */

pub struct DipSwitch {
    gpios: (
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
    ),
}

#[allow(dead_code)]
impl DipSwitch {
    pub const fn new(
        in_price0: Input<'static, AnyPin>,
        in_price1: Input<'static, AnyPin>,
        in_timing0: Input<'static, AnyPin>,
        in_timing1: Input<'static, AnyPin>,
        in_mode0: Input<'static, AnyPin>,
        in_mode1: Input<'static, AnyPin>,
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
                self.gpios.0.is_high() as u8 + self.gpios.1.is_high() as u8 * 2,
            )
            .unwrap(),
            TimingOverride::try_from(
                self.gpios.2.is_high() as u8 + self.gpios.3.is_high() as u8 * 2,
            )
            .unwrap(),
            AppMode::try_from(self.gpios.4.is_high() as u8 + self.gpios.5.is_high() as u8 * 2)
                .unwrap(),
        )
    }
}
