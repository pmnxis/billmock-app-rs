/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use core::cell::UnsafeCell;
// https://docs.rust-embedded.org/book/concurrency/

#[derive(Debug, Clone, Copy)]
pub struct ToggleTiming {
    pub high_ms: u16,
    pub low_ms: u16,
}

impl Default for ToggleTiming {
    fn default() -> Self {
        // default is 100ms high low signal.
        ToggleTiming {
            high_ms: 100,
            low_ms: 100,
        }
    }
}

#[derive(Debug)]
pub struct SharedToggleTiming(UnsafeCell<ToggleTiming>);

impl SharedToggleTiming {
    pub fn default(&self) {
        unsafe { *self.0.get() = ToggleTiming::default() };
    }

    pub fn set(&self, value: ToggleTiming) {
        unsafe { *self.0.get() = value };
    }

    pub fn get(&self) -> ToggleTiming {
        unsafe { *self.0.get().clone() }
    }
}

// Required to allow static SharedToggleTiming
// see : https://docs.rust-embedded.org/book/concurrency/#abstractions-send-and-sync
unsafe impl Sync for SharedToggleTiming {}

pub struct DualPoleToggleTiming {
    /// shared toggle timing, but not guaranteed ordering and something else.
    pub shared: &'static SharedToggleTiming,
    /// prefer const-ish value (todo tided on const only)
    /// alt field is not allowed modification on runtime.
    pub alt: &'static ToggleTiming,
}
