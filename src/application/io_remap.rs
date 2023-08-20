/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use crate::boards::*;
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::input_port::{InputEvent, InputPortKind};

#[derive(Debug, Clone, Copy)]
pub enum InsteadStartJam {
    Start,
    Jam,
}

impl InputEvent {
    pub fn remap(&mut self, instead: InsteadStartJam) {
        match (self.port, instead) {
            (InputPortKind::StartJam1P, InsteadStartJam::Jam) => {
                self.port = InputPortKind::Jam1P;
            }
            (InputPortKind::StartJam2P, InsteadStartJam::Jam) => {
                self.port = InputPortKind::Jam2P;
            }
            (InputPortKind::StartJam1P, InsteadStartJam::Start) => {
                self.port = InputPortKind::Start1P;
            }
            (InputPortKind::StartJam2P, InsteadStartJam::Start) => {
                self.port = InputPortKind::Start2P;
            }
            _ => {}
        }
    }
}
