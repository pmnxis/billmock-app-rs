/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;

use crate::semi_layer::buffered_opendrain::BufferedOpenDrain;
use crate::semi_layer::buffered_wait::{BufferedWait, InputEventChannel, InputPortKind};
use crate::semi_layer::timing::DualPoleToggleTiming;

pub struct VendSideBill<OutInhibit: OutputPin, InVend: Wait, InJam: Wait> {
    out_inhibit: BufferedOpenDrain<OutInhibit>,
    in_vend: BufferedWait<InVend>,
    in_jam: BufferedWait<InJam>,
}

impl<OutInhibit: OutputPin, InVend: Wait, InJam: Wait> VendSideBill<OutInhibit, InVend, InJam> {
    pub fn new(
        mut out_inhibit: OutInhibit,
        (in_vend, in_vend_event): (InVend, InputPortKind),
        (in_jam, in_jam_event): (InJam, InputPortKind),
        mpsc_ch: &'static InputEventChannel,
        timing: &'static DualPoleToggleTiming,
    ) -> VendSideBill<OutInhibit, InVend, InJam> {
        out_inhibit.set_low().ok();

        Self {
            out_inhibit: BufferedOpenDrain::new(out_inhibit, timing),
            in_vend: BufferedWait::new(in_vend, in_vend_event, mpsc_ch),
            in_jam: BufferedWait::new(in_jam, in_jam_event, mpsc_ch),
        }
    }
}
