<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Hardware 

| top side | bottom side |
| ---- | ---- |
| ![top side image](https://billmock.gpark.biz/images/pcb_top_0v4.png) | ![bottom side image](https://billmock.gpark.biz/images/pcb_bot_0v4.png) | 

This https://billmock.gpark.biz/images are 0.4 hardware 3D rendering

## Specifications

|             |              |
| ----------- | ------------ |
| Product name| BillMock     |
| Manufacturer| GPARK Co., Ltd. |
| Country     | South Korea |
| Dimension   | 65.0 mm * 65.0 mm |
| MCU         | STM32G030C8 |
| MCU Spec    | ARM Cortex-M0+ 64Mhz CPU, 8KiB SRAM, 64KiB Flash |
| Software    | Embassy-rs written in rust |
| Power Input | 12V 2A |
| Pouwer Output | 5V (Peak 2.2A 300mS trip, 1.1A nominal MAX) - Credit card reader power |

※ The input power allows up to 16V, but please be cautious as it is directly passed through to the bill handling device.

## Hardware design
BillMock hardware schematic repository (only pdf) : [BillMock Hardware PDF Release](https://github.com/pmnxis/BillMock-HW-RELEASE)

The schematic printed in PDF is distributed under CC BY-SA 3.0, but the actual Gerber files and project files are private.

#### v 0.2 (2023-06-13)
[BillMock-HW-0v2.pdf](https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-HW-0v2.pdf)

#### v 0.3 (2023-08-11)
[BillMock-HW-0v3.pdf](https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-HW-0v3.pdf)

#### v 0.4 (2023-08-30)
[BillMock-HW-0v4.pdf](https://github.com/pmnxis/BillMock-HW-RELEASE/blob/master/sch/BillMock-HW-0v4.pdf)