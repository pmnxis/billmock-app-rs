/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use static_assertions::*;

#[repr(packed(2))]
#[derive(Clone)]
pub struct FaultLog {
    pub current_boot_cnt: u32,
    pub error_code: u16,
}
assert_eq_size!(FaultLog, [u8; 6]);
