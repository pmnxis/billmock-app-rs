<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Vend side port map

## Vend Side Quick Terminal
<table>
<tr>
<td>

![J9](https://billmock.gpark.biz/images/pcb_0v4_port/J9.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J9  |
|                | Existing coin and bill acceptor ports (terminal types) |
| Connector      | 141R-2.54-8P |

| **Pin #** | **Pin Name** | Annotation                               |
| :-------: | ------------ | ---------------------------------------- |
| `1`       | `GND`        | Bill acceptor - Pole Power (Input/Output) |
| `2`       | `GND`        | Bill acceptor - Pole Power (Input/Output) |
| `3`       | `12V`        | Bill acceptor + Pole Power (Input/Output) |
| `4`       | `INHIBIT`    | Bill acceptor Inhibit Deactivate Output Signal |
| `5`       | `JAM`        | Bill acceptor 1P Coin Insertion Signal         |
| `6`       | `VEND`       | Bill acceptor 2P Coin Insertion Signal         |
| `7`       | `START1`     | Start Button 1 Input Signal                    |
| `8`       | `START2`     | Start Button 2 Input Signal                    |

</td></tr>
</table>

- Pins are counted from the left.
- You can also input power directly into BillMock-HW through `12V` and `GND` pins.
- While it's more convenient to strip tde insulation from tde cable in tde middle and tden connect it to tde terminal,
- It's recommended, whenever possible, to use a cable type that comes pre-equipped for the connection.
- START1/2 can be changed to JAM based on DIP switch settings

------------

## Vend Side Player 1 Port (left)
<table>
<tr>
<td>

![J7](https://billmock.gpark.biz/images/pcb_0v4_port/J7.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J7  |
|                | Player 1 side existing <br/>coin and bill acceptor port |
| Connector      | Molex 53014-10xx |
| Housing        | Molex 51004-10xx |
| Crimp          | Molex 50011 |

| **Pin #** | **Pin Name** | Annotation                                     |
| :-------: | ------------ | ---------------------------------------------- |
| `1`       | N/C          | Not Connected                                  |
| `2`       | `VEND`       | Bill acceptor Coin Insertion Signal      |
| `3`       | N/C          | Not Connected                                  |
| `4`       | `START`      | Start Button Input Signal                      |
| `5`       | `INHIBIT`    | Bill acceptor Inhibit (Deactivate) Output Signal |
| `6`       | `GND`        | Bill acceptor - Pole Power (Input/Output) |
| `7`       | N/C          | Not Connected                             |
| `8`       | `12V`        | Bill acceptor + Pole Power (Input/Output) |
| `9`       | `12V`        | Bill acceptor + Pole Power (Input/Output) |
| `10`      | `GND`        | Bill acceptor - Pole Power (Input/Output) |

</td></tr>
</table>

- Pins are counted from the left.
- START can be changed to JAM based on DIP switch settings

------------

## Vend Side Player 2 Port (right)
<table>
<tr>
<td>

![J8](https://billmock.gpark.biz/images/pcb_0v4_port/J8.png)
</td>
<td>

|                |                |
| -------------- | -------------- |
| Designator     | J8  |
|                | Player 2 side existing <br/>coin and bill acceptor port |
| Connector      | Molex 53014-10xx |
| Housing        | Molex 51004-10xx |
| Crimp          | Molex 50011 |

| **Pin #** | **Pin Name** | Annotation                                     |
| :-------: | ------------ | ---------------------------------------------- |
| `1`       | N/C          | Not Connected                                  |
| `2`       | `VEND`       | Bill acceptor Coin Insertion Signal      |
| `3`       | N/C          | Not Connected                                  |
| `4`       | `START`      | Start Button Input Signal                      |
| `5`       | `INHIBIT`    | Bill acceptor Inhibit (Deactivate) Output Signal |
| `6`       | `GND`        | Bill acceptor - Pole Power (Input/Output) |
| `7`       | N/C          | Not Connected                             |
| `8`       | `12V`        | Bill acceptor + Pole Power (Input/Output) |
| `9`       | `12V`        | Bill acceptor + Pole Power (Input/Output) |
| `10`      | `GND`        | Bill acceptor - Pole Power (Input/Output) |

</td></tr>
</table>

- Pins are counted from the left.
- START can be changed to JAM based on DIP switch settings