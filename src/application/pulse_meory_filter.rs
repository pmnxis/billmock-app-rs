use card_terminal_adapter::CardTerminalTxCmd;
use embassy_time::{Duration, Instant};

use crate::boards::*;
use crate::components::eeprom;
use crate::semi_layer::buffered_wait::InputEventKind;
use crate::types::input_port::{InputEvent, InputPortKind};

#[derive(Clone)]
pub(crate) struct PulseMemory {
    pub last_tick: Instant,
    pub deadline: Duration,
    pub count: Option<u16>,
}

pub(crate) struct PulseMemoryFilterMachine {
    pub player: [PulseMemory; PLAYER_INDEX_MAX],
}

impl PulseMemory {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            deadline: Duration::from_millis(200),
            count: None,
        }
    }

    pub fn is_overtime(&self) -> bool {
        (self.last_tick + self.deadline) < Instant::now()
    }

    pub fn reset(&mut self) {
        self.count = None;
        self.deadline = Duration::from_millis(200); // this should be set
    }

    pub fn is_stopped(&self) -> bool {
        self.count.is_none()
    }

    pub fn is_running(&self) -> bool {
        self.count.is_some()
    }

    pub fn is_running_and_overtime(&self) -> bool {
        self.is_running() && self.is_overtime()
    }

    pub fn mark(&mut self, timing_in_ms: u16) {
        let was_stopped = self.is_stopped();
        self.count = Some(self.count.unwrap_or_default() + 1);

        self.last_tick = Instant::now();
        if was_stopped {
            let deadline_in_ms = ((timing_in_ms * 3) / 2).min(1200);
            self.deadline = Duration::from_millis(deadline_in_ms.into());
        }
    }
}

impl PulseMemoryFilterMachine {
    pub fn new() -> Self {
        let new = PulseMemory::new();
        Self {
            player: [new.clone(), new],
        }
    }

    #[allow(unused)] // this is just example of usage.
    pub fn detect_signal(&mut self, input: &InputEvent) {
        // should I detect signal first?
        if let Some((index, timing_in_ms)) = match input {
            InputEvent {
                port: InputPortKind::Vend1P,
                event: InputEventKind::LongPressed(time_in_10ms),
            } => Some((PLAYER_1_INDEX, (*time_in_10ms as u16) * 10)),
            InputEvent {
                port: InputPortKind::Vend2P,
                event: InputEventKind::LongPressed(time_in_10ms),
            } => Some((PLAYER_2_INDEX, (*time_in_10ms as u16) * 10)),
            _ => None,
        } {
            let mut target = &mut self.player[index];
            target.mark(timing_in_ms);
        }
    }

    pub async fn report_when_expired(&mut self, board: &'static Board) {
        for player_index in [PLAYER_1_INDEX, PLAYER_2_INDEX] {
            if self.player[player_index].is_running_and_overtime() {
                if let Some(assume_report) = board
                    .hardware
                    .eeprom
                    .lock_read(eeprom::select::CARD_PORT_BACKUP)
                    .await
                    .guess_raw_income_by_player(1 + player_index as u8)
                {
                    let actual = self.player[player_index].count.unwrap_or_default();
                    let expected = assume_report.get_pulse_count();
                    if actual != expected {
                        defmt::warn!("Player {} - CashReceipt clock mismatch but ignored, actual : {}, expected : {}", player_index+1, actual, expected);
                    } else {
                        defmt::info!(
                            "Player {} - CashReceipt clock actual : {}",
                            player_index + 1,
                            actual
                        );
                    }

                    board
                        .hardware
                        .card_reader
                        .send(CardTerminalTxCmd::PushCoinPaperAcceptorIncome(
                            assume_report.clone(),
                        ))
                        .await;
                } else {
                    defmt::info!(
                        "Player {} - cash receipt could not be requested",
                        player_index + 1
                    );
                }

                self.player[player_index].reset();
            }
        }
    }
}
