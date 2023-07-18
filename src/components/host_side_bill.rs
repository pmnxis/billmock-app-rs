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

pub struct HostSideBill<
    InInhibit: Wait,
    OutBusy: OutputPin,
    OutVend: OutputPin,
    OutJam: OutputPin,
    OutStart: OutputPin,
> {
    in_inhibit: BufferedWait<InInhibit>,
    out_busy: BufferedOpenDrain<OutBusy>,
    out_vend: BufferedOpenDrain<OutVend>,
    out_jam: BufferedOpenDrain<OutJam>,
    out_start: BufferedOpenDrain<OutStart>,
}

impl<
        InInhibit: Wait,
        OutBusy: OutputPin,
        OutVend: OutputPin,
        OutJam: OutputPin,
        OutStart: OutputPin,
    > HostSideBill<InInhibit, OutBusy, OutVend, OutJam, OutStart>
{
    pub fn new(
        (in_inhibit, in_inhibit_event): (InInhibit, InputPortKind),
        mut out_busy: OutBusy,
        mut out_vend: OutVend,
        mut out_jam: OutJam,
        mut out_start: OutStart,
        mpsc_ch: &'static InputEventChannel,
        timing: &'static DualPoleToggleTiming,
    ) -> HostSideBill<InInhibit, OutBusy, OutVend, OutJam, OutStart> {
        Self {
            in_inhibit: BufferedWait::new(in_inhibit, in_inhibit_event, mpsc_ch),
            out_busy: BufferedOpenDrain::new(out_busy, timing),
            out_vend: BufferedOpenDrain::new(out_vend, timing),
            out_jam: BufferedOpenDrain::new(out_jam, timing),
            out_start: BufferedOpenDrain::new(out_start, timing),
        }
    }
}
