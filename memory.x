/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

/* STM32G030C8 */
MEMORY
{
  /*
   * [Mass Production Reminder]
   * It is feasible to consider switching to the STM32G030C6Tx chip
   * for the purpose of addressing supply availability and cost reduction.
   * However, please be aware that the current debug build exceeds the 32KB
   * limit for the flash section.
   * As a result, this change would be applicable only when transitioning
   * to the release build for mass production.
   *
   * If you wish to proceed with the change to STM32G030C6Tx,
   * please modify the FLASH LENGTH to 32KB.
  */
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 8K
}
