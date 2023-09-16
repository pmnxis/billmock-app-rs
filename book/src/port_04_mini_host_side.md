<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Host side 핀 아웃

## Host Side Player 1 Port (left)

![J3](https://billmock.gpark.biz/images/pcb_0v4_mini_port/J3.png)

|                |                |
| -------------- | -------------- |
| Designator     | J3  |
|                | Emulated Player 1 side connector towards the GAME IO side |
| Connector      | 141R-2.54-8P |

| **Pin #** | **Pin Name**   | Annotation                                             |
| :-------: | -------------- | ------------------------------------------------------ |
| `1`       | `V1-BUSY`      | Emulated Busy state Output Signal                      |
| `2`       | `V1-VEND`      | Emulated Bill acceptor 1P Coin Insertion Output Signal |
| `3`       | `V1-JAM`       | Emulated JAM Output Signal                             |
| `4`       | `V1-START`     | Emulated 1P Start Button Output Signal                 |
| `5`       | `V1-INHIBIT`   | Inhibit Input Signal from 1P GAME I/O                  |
| `6`       | N/C            | Not Connected                                          |
| `7`       | N/C            | Not Connected                                          |
| `8`       | `12V`          | Product + Pole Power Input                             |
| `9`       | `12V`          | Product + Pole Power Input                             |
| `10`      | `GND`          | Product - Pole Power Input                             |


- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- The power pins from this port cannot be used for power output. When power input to this port is blocked, reverse voltage does not flow.
- The "Busy" output signal remains active low from the moment a payment signal is received from the credit card or when the VEND input signal goes active low until the VEND output signal toggles and completes.

------------

## Host Side Player 2 Port (right)

![J2](https://billmock.gpark.biz/images/pcb_0v4_mini_port/J2.png)

|                |                |
| -------------- | -------------- |
| Designator     | J2  |
|                | Emulated Player 2 side connector towards the GAME IO side |
| Connector      | 141R-2.54-8P |

| **Pin #** | **Pin Name**   | Annotation                                             |
| :-------: | -------------- | ------------------------------------------------------ |
| `1`       | `V1-BUSY`      | Emulated Busy state Output Signal                      |
| `2`       | `V1-VEND`      | Emulated Bill acceptor 2P Coin Insertion Output Signal |
| `3`       | `V1-JAM`       | Emulated JAM Output Signal                             |
| `4`       | `V1-START`     | Emulated 2P Start Button Output Signal                 |
| `5`       | `V1-INHIBIT`   | Inhibit Input Signal from 2P GAME I/O                  |
| `6`       | N/C            | Not Connected                                          |
| `7`       | N/C            | Not Connected                                          |
| `8`       | `12V`          | Product + Pole Power Input                             |
| `9`       | `12V`          | Product + Pole Power Input                             |
| `10`      | `GND`          | Product - Pole Power Input                             |

- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- The power pins from this port cannot be used for power output. When power input to this port is blocked, reverse voltage does not flow.
- The "Busy" output signal remains active low from the moment a payment signal is received from the credit card or when the VEND input signal goes active low until the VEND output signal toggles and completes.
