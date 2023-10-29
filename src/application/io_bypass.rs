/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use defmt::{error, warn};

use super::{DEFAULT_BUSY_ALPHA_TIMING_MS, DEFAULT_VEND_INDICATOR_TIMING_MS};
use crate::boards::*;
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::input_port::{InputEvent, InputPortKind};

pub async fn io_bypass(board: &'static Board, event: &InputEvent, override_druation_force: bool) {
    let output = match board.correspond_output(&event.port) {
        Ok(x) => x,
        #[cfg(feature = "svc_button")]
        Err(BoardCorrespondOutputMatchError {
            origin: InputPortKind::SvcButton,
        }) => {
            // Svc Button is not output type
            return;
        }
        Err(e) => {
            error!("io_bypass some wrong enum value income : {}", e.origin);
            return;
        }
    };

    let led = board.correspond_indicator(&event.port);
    let busy = board.correspond_busy(&event.port);

    match (event.port, event.event) {
        (x, InputEventKind::LongPressed(0) | InputEventKind::LongPressed(1)) => {
            warn!("{:?} too short pressed", x);
        }
        (InputPortKind::Vend1P | InputPortKind::Vend2P, InputEventKind::LongPressed(x)) => {
            let led_timing = if override_druation_force {
                output.tick_tock(1).await;

                DEFAULT_VEND_INDICATOR_TIMING_MS
            } else {
                let mul10 = (x as u16) * 10;
                output.alt_tick_tock(1, mul10, mul10).await;

                mul10
            };

            if let Some(busy) = busy {
                busy.one_shot_high_mul(1, led_timing, led_timing, DEFAULT_BUSY_ALPHA_TIMING_MS)
                    .await;
            }

            if let Some(led) = led {
                led.alt_tick_tock(1, led_timing, led_timing).await;
            }
        }
        (InputPortKind::StartJam1P | InputPortKind::StartJam2P, _) => {
            // skip
        }
        (_, InputEventKind::Pressed) => {
            output.set_high().await;
        }
        (_, InputEventKind::Released) => {
            output.set_low().await;
        }
        (_, InputEventKind::LongPressed(_)) => {
            // skip
        }
    }
}
