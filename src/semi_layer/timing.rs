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

impl ToggleTiming {
    pub const fn default() -> Self {
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
    pub const fn new_custom(timing: ToggleTiming) -> Self {
        // https://doc.rust-lang.org/reference/const_eval.html#const-functions
        Self(UnsafeCell::new(timing))
    }

    pub const fn default() -> Self {
        Self(UnsafeCell::new(ToggleTiming::default()))
    }

    #[allow(dead_code)]
    pub fn set(&self, value: ToggleTiming) {
        unsafe { *self.0.get() = value };
    }

    #[allow(dead_code)]
    pub fn get(&self) -> ToggleTiming {
        unsafe { *self.0.get() }
    }
}

// Required to allow static SharedToggleTiming
// see : https://docs.rust-embedded.org/book/concurrency/#abstractions-send-and-sync
unsafe impl Sync for SharedToggleTiming {}
