/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use defmt::export::panic;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;
use {defmt_rtt as _, panic_probe as _};

pub struct VendSideBill<OUT_INHIBIT: OutputPin, IN_VEND: Wait, IN_JAM: Wait> {
    // hardware related
    out_inhibit: OUT_INHIBIT,
    in_vend: IN_VEND,
    in_jam: IN_JAM,
}

impl<OUT_INHIBIT: OutputPin, IN_VEND: Wait, IN_JAM: Wait>
    VendSideBill<OUT_INHIBIT, IN_VEND, IN_JAM>
{
    pub fn new(
        mut out_inhibit: OUT_INHIBIT,
        in_vend: IN_VEND,
        in_jam: IN_JAM,
    ) -> VendSideBill<OUT_INHIBIT, IN_VEND, IN_JAM> {
        // Ensure 5 pins are initialized.
        out_inhibit.set_low().ok();

        VendSideBill {
            out_inhibit,
            in_vend,
            in_jam,
        }
    }
}
