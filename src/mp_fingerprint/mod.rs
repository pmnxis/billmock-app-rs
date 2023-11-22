/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//! This code is not reflected on program binary or actual memory.
//! .mp_fingerprint section is `NOLOAD` and `READONLY` section that
//! virtually existed in ELF header.
//! The contents of .mp_fingerprint would be used for billmock-mptool
//! to determine what kind of board, feature, version, git hash based
//! from ELF binary.

//! DO NOT USE `&str` or any other slice type
//! ```rs
//! #[allow(unused, dead_code)]
//! #[no_mangle]
//! #[link_section = ".mp_fingerprint"]
//! static TEST_FINGER: [u8; 14] = *b"SOME TOML HERE";
//! ```

use env_to_array::patch_linker_section_from_hex_env;

patch_linker_section_from_hex_env!(".mp_fingerprint", "MP_INFO_TOML", "MP_FINGERPRINT_TOML_HEX");
