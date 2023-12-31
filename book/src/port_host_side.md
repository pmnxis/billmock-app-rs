<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Host side port map

## Host Side Quick Terminal
<table>
<tr>
<td>

![J6](https://billmock.gpark.biz/images/pcb_0v4_port/J6.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J6  |
|                | terminal that emulates towards the GAME IO side |
| Connector      | 141R-2.54-8P |

| **Pin #** | **Pin Name**   | Annotation                                                |
| :-------: | -------------- | --------------------------------------------------------- |
| `1`       | `GND`          | Product - Pole Power Input (Add' Output) |
| `2`       | `GND`          | Product - Pole Power Input (add' Output) |
| `3`       | `12V`          | Product + Pole Power Input                               |
| `4`       | `V1-INHIBIT`   | Inhibit Input Signal from 1P GAME I/O |
| `5`       | `V1-VEND`      | Emulated Bill acceptor 1P Coin Insertion Output Signal |
| `6`       | `V2-VEND`      | Emulated Bill acceptor 2P Coin Insertion Output Signal |
| `7`       | `V1-START`     | Emulated 1P Start Button Output Signal |
| `8`       | `V2-START`     | Emulated 2P Start Button Output Signal |

</td></tr>
</table>

- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- While it's more convenient to strip tde insulation from tde cable in tde middle and tden connect it to tde terminal,
- It's recommended, whenever possible, to use a cable type that comes pre-equipped for the connection.

------------

## Host Side Player 1 Port (left)
<table>
<tr>
<td>

![J5](https://billmock.gpark.biz/images/pcb_0v4_port/J5.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J5  |
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

</td></tr>
</table>

- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- The power pins from this port cannot be used for power output. When power input to this port is blocked, reverse voltage does not flow.
- The "Busy" output signal remains active low from the moment a payment signal is received from the credit card or when the VEND input signal goes active low until the VEND output signal toggles and completes.

------------

## Host Side Player 2 Port (right)
<table>
<tr>
<td>

![J4](https://billmock.gpark.biz/images/pcb_0v4_port/J4.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J4  |
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

</td></tr>
</table>

- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- The power pins from this port cannot be used for power output. When power input to this port is blocked, reverse voltage does not flow.
- The "Busy" output signal remains active low from the moment a payment signal is received from the credit card or when the VEND input signal goes active low until the VEND output signal toggles and completes.
