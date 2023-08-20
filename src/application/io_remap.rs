/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use super::io_bypass::io_bypass;
use crate::boards::*;
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::input_port::{InputEvent, InputPortKind};

#[derive(Debug, Clone, Copy)]
pub enum InsteadStartJam {
    Start,
    Jam,
}

impl InputEvent {
    pub fn remap(&mut self, instead: InsteadStartJam) -> Self {
        let port = match (self.port, instead) {
            (InputPortKind::StartJam1P, InsteadStartJam::Jam) => InputPortKind::Jam1P,
            (InputPortKind::StartJam2P, InsteadStartJam::Jam) => InputPortKind::Jam2P,
            (InputPortKind::StartJam1P, InsteadStartJam::Start) => InputPortKind::Start1P,
            (InputPortKind::StartJam2P, InsteadStartJam::Start) => InputPortKind::Start2P,
            (x, _) => x,
        };

        Self {
            port,
            event: self.event,
        }
    }

    pub fn replace(&self, port: (InputPortKind, InputPortKind)) -> Self {
        if self.port == port.0 {
            Self {
                port: port.1,
                event: self.event,
            }
        } else {
            *self
        }
    }

    pub fn replace_arr(&self, ports: &[(InputPortKind, InputPortKind)]) -> Self {
        for port in ports {
            if self.port == port.0 {
                return Self {
                    port: port.1,
                    event: self.event,
                };
            }
        }
        *self
    }

    pub fn ignore(&self, port: InputPortKind) -> Self {
        // todo! - bitfield based filter system efficient instruction usage
        if self.port == port {
            Self {
                port: InputPortKind::Nothing,
                event: self.event,
            }
        } else {
            *self
        }
    }

    pub fn ignore_arr(&self, ports: &[InputPortKind]) -> Self {
        // todo! - bitfield based filter system efficient instruction usage
        for port in ports {
            if self.port == *port {
                return Self {
                    port: InputPortKind::Nothing,
                    event: self.event,
                };
            }
        }
        *self
    }

    pub fn allow(&self, port: InputPortKind) -> Self {
        // todo! - bitfield based filter system efficient instruction usage
        if self.port == port {
            *self
        } else {
            Self {
                port: InputPortKind::Nothing,
                event: self.event,
            }
        }
    }

    pub fn allow_arr(&self, ports: &[InputPortKind]) -> Self {
        // todo! - bitfield based filter system efficient instruction usage
        for port in ports {
            if self.port == *port {
                return *self;
            }
        }
        Self {
            port: InputPortKind::Nothing,
            event: self.event,
        }
    }

    pub async fn apply_output(&self, board: &'static Board) -> Self {
        io_bypass(board, self).await;
        *self
    }
}
