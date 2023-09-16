<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Application Logic

The simplified program and signal input-output flow are as follows: </br>

![BillMock Diagram](https://billmock.gpark.biz/images/billmock_logic_diagram_short.png)


## Overview of Operation
This device is designed to enhance payment systems by storing, manipulating, and delaying input/output signals from currency payment devices (such as credit card readers, coin acceptors, bill validators) before connecting them to the GAME I/O PCB.

By configuring DIP switch settings and making appropriate wiring changes, you can guide the setup between the desired currency payment device and the GAME I/O PCB. This device enables the following configurations:

- Installing a credit card reader on an existing game machine.
- Using both a credit card reader and a bill validator (or coin acceptor) on 1P (Player 1) in an existing game machine.
- Managing 1P/2P (Player 1/Player 2) when using one card reader and one bill validator (or coin acceptor) on an existing game machine upon pressing the start button.
- Managing 1P/2P (Player 1/Player 2) when using one bill validator (or coin acceptor) on an existing game machine upon pressing the start button.
- Override output pulse duration based DIP switch configuration and input signal.
- Display accumulated card / paper(or coin) count instead of magnetic coin meter.
- Allowing users to make more complex modifications to input/output signals through code customization.
