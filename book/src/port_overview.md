<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Difference between two hardware types

| Mini types | regular version 버젼 |
| ---- | ---- |
| ![mini image](https://billmock.gpark.biz/images/pcb_top_mini_0v4.png) | ![normal image](https://billmock.gpark.biz/images/pcb_top_0v4.png) |

BillMock hardware comes in two different variants. There is the smaller rectangular-shaped [BillMock Mini (Rectangular)](./port_04_mini_overview.md) and the square-shaped [BillMock (Square)](./port_04_overview.md) with terminals and a DC jack on the top and bottom.

The Mini version removes the terminals, DC jack, and additional RS232 port compared to the standard version. It comes at a lower price point and is easier to install if you already have the harness prepared. In situations where harness configuration is complex and it's more convenient to connect directly to the terminals on-site, the standard version may be suitable.

However, for most on-site installations, we recommend the Mini version as it offers greater efficiency, especially when using pre-configured harnesses, making the installation process smoother.

## Table of Contents

- [BillMock Mini (Rectangular)](./port_04_mini_overview.md)
    - [Vend side (Top)](./port_04_mini_vend_side.md)
    - [Host Side (Bottom)](./port_04_mini_host_side.md)
    - [Miscellaneous](./port_04_mini_etc.md)
- [BillMock (Square)](./port_04_overview.md)
    - [Vend side (Top)](./port_vend_side.md)
    - [Host Side (Bottom)](./port_host_side.md)
    - [Miscellaneous](./port_etc.md)
    
At the top, there are connectors for the Vend Side, which includes bill paper acceptors, coin acceptors, credit card reader, and similar currency validators.

At the bottom, connectors are placed for the Host Side, which interfaces with the mainboard of the actual arcade machine, such as the GAME I/O PCB.

On the left and right sides, you'll find identical connectors arranged in a decalcomania pattern. The connectors on the left are designated for Player 1, while those on the right are for Player 2.

With this pattern of connectors, it's easy to connect wires to the connectors during actual installation and operation by referring to the layout.
