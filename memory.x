/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

/* STM32G030C8 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 8K
}

/*
 * Mass-Production usage ELF section
 * ref - https://github.com/pmnxis/billmock-app-rs/issues/40
 * ref - https://sourceware.org/binutils/docs/ld/Output-Section-Type.html
 */
SECTIONS {
  .mp_fingerprint 0 (OVERLAY) :
  {
    KEEP(*(.mp_fingerprint))
  }
}
