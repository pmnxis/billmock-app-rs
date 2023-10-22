/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use billmock_otp_dev_info::OtpDeviceInfo;
use env_to_array::hex_env_to_array;

pub const PROJECT_NAME: &str = env!("PROJECT_NAME");

pub const VERSION_STR: [u8; card_terminal_adapter::FW_VER_LEN] =
    hex_env_to_array!("PROJECT_VERSION");

pub const COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");

pub(crate) const COMMIT_SHORT: [u8; card_terminal_adapter::GIT_HASH_LEN] =
    hex_env_to_array!("GIT_COMMIT_SHORT_HASH");

pub const SERIAL_NUMBER_WHEN_UNKNOWN: [u8; card_terminal_adapter::DEV_SN_LEN] = *b"     unknown";

pub const GIT_COMMIT_DATETIME: &str = env!("GIT_COMMIT_DATETIME");
pub const PRINT_BAR: &str = "+-----------------------------------------------------------+";

pub fn get_serial_number() -> &'static [u8; card_terminal_adapter::DEV_SN_LEN] {
    let otp_space = OtpDeviceInfo::from_stm32g0();

    match otp_space.check_and_sn() {
        Err(_) => &SERIAL_NUMBER_WHEN_UNKNOWN,
        Ok(_) => &otp_space.dev_sn,
    }
}
