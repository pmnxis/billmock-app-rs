<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# BillMock

![real photo](https://billmock.gpark.biz/images/pcb_0v4_port/BillMockPCB_0v4_square.jpg)

| Front side | Back side |
| ---- | ---- |
| ![top side image](https://billmock.gpark.biz/images/pcb_top_0v4.png) | ![bottom side image](https://billmock.gpark.biz/images/pcb_bot_0v4.png) | 

## Pin map overview

![Port Quick Look](https://billmock.gpark.biz/images/pcb_0v4_port/port_quick_look.png)

At the top, there are connectors for the Vend Side, which includes bill paper acceptors, coin acceptors, credit card reader, and similar currency validators.

At the bottom, connectors are placed for the Host Side, which interfaces with the mainboard of the actual arcade machine, such as the GAME I/O PCB.

On the left and right sides, you'll find identical connectors arranged in a decalcomania pattern. The connectors on the left are designated for Player 1, while those on the right are for Player 2.

With this pattern of connectors, it's easy to connect wires to the connectors during actual installation and operation by referring to the layout.

From a conceptual perspective, in the existing wiring, the connectors that were originally connected from top to bottom are each separated and connected to the upper and lower connectors. The hardware and software manage the communication between these upper and lower connectors. This design was intentional during the connection setup.

In addition to these connectors, there are connectors for power supply, an additional RS232 connector for debugging, and an SWD (JTAG) connector for program debugging.

## Simplified Connection of BillMock
![Simplified Wiring](https://billmock.gpark.biz/images/pcb_0v4_port/wiring.png)

- WARN : The wire shapes are symbolic representations and do not indicate the actual wire colors used in the image. Please refer to detailed pinouts in the respective pages.

This configuration involves installing this PCB (BillMock) between the existing bill acceptor (or coin acceptor) on the top and the GAME I/O PCB on the bottom, with a harness connection.

In some cases, additional wiring work may be required, but considering the widely used bill acceptor wiring in South Korea, you can prepare the harness and connect it accordingly.

Unlike the BillMock Mini version, you can also perform wiring work through terminals. For more details, please refer to the [Machine Installation](./installation.md) document.

Additionally, the terminal pinouts are slightly different from standard Molex connectors, so please review this document for details on the top and bottom terminals:

- [Vend side (Top) Terminal](./port_vend_side.md#vend-side-quick-terminal)
- [Host Side (Bottom) Terminal](./port_host_side.md#host-side-quick-terminal)

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

- WARN : The input power allows up to 16V, but please be cautious as it is directly passed through to the bill handling device.
