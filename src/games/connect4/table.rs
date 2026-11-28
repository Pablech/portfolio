use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::games::{Agent, GameError, GameLogic, connect4::COMBS};

const CELLS: usize = 42;
const LAST: &str = "red";

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)] 
pub struct Conn4Table {
    #[serde_as(as = "[_; CELLS]")] 
    pub cells: [Option<u8>; CELLS],
    pub last_mv: Option<usize>,
}

impl Default for Conn4Table {
    fn default() -> Self {
        Self { cells: [None; CELLS], last_mv: None, }
    }
}

impl Conn4Table {
    fn cell_indx(row: usize, col: usize) -> usize {
        row * 7 + col
    }

    fn col_free(&self, col: usize) -> bool {
        self.cells[Self::cell_indx(0, col)].is_none()
    }

    pub fn is_win_with(&self, mv: usize, player: u8) -> bool {
        let mut sim = self.clone();
        if sim.place(mv, player).is_ok() {
            sim.check_win(player)
        } else {
            false
        }
    }
}

impl GameLogic for Conn4Table {
    const SEATS: usize = 2;
    
    fn new() -> Self where Self: Sized {
        Self::default()
    }

    fn legal_moves(&self) -> Vec<usize> {
        (0..7).filter(|&c| self.col_free(c)).collect()
    }
    
    fn place(&mut self, mv: usize, player: u8) -> Result<(), GameError> {
        if !self.col_free(mv) {
            return Err(GameError::ErrorMove("Coluna cheia".to_string()));
        }

        for row in (0..6).rev() {
            let indx = Self::cell_indx(row, mv);
            if self.cells[indx].is_none() {
                self.cells[indx] = Some(player);
                self.last_mv = Some(row * 7 + mv);
                return Ok(());
            }
        }
        unreachable!()
    }

    fn check_win(&self, fig: u8) -> bool {
        for comb in COMBS.iter() {
            if comb.iter().all(|&i| self.cells[i] == Some(fig)) {
                return true;
            }
        }
        false
    }

    fn check_draw(&self) -> bool {
        self.cells.iter().all(|c| c.is_some())
    }

    fn output(&self, players: &[Agent], seat: &Option<usize>) -> String {

        let _ = seat;

        let fig_of = |c: Option<u8>, last: bool| -> String {
            let fig = match c {
                None => format!("{:^3}", " "),
                Some(1) => format!("{:^3}", players[0].fig),
                Some(2) => format!("{:^3}", players[1].fig),
                _ => "???".to_string(),
            };
            if last {
                format!("<span style=\"color:{LAST};\">{fig}</span>")
            } else {
                fig.to_string()
            }
        };

        let mut board = String::new();
        for row in 0..6 {
            board.push('|');
            for col in 0..7 {
                let indx = row * 7 + col;
                let last = self.last_mv == Some(indx);
                board.push_str(&fig_of(self.cells[indx], last));
                board.push('|');
            }
            board.push('\n');
        }

        format!(
            "JOGADOR 1: {} = {}\nJOGADOR 2: {} = {}\n  1   2   3   4   5   6   7\n{}\n",
            players[0].name, players[0].fig,
            players[1].name, players[1].fig,
            board
        )
    }

    fn shared_output(&self) -> bool {
        true
    }

    fn parse_input(input: &str) -> Result<usize, GameError> {
        let mv: usize = input.trim().parse::<usize>()
            .map_err(|_| GameError::ErrorMove(format!("entrada {} inválida", input)))?;

        let mv = mv.checked_sub(1)
            .ok_or_else(|| GameError::ErrorMove("coluna começa em 1".into()))?;
        Ok(mv)
    }

    fn move_label(mv: usize) -> String {
        format!("{}", mv + 1)
    }

    fn next_turn(&self, current_turn: usize, num_seats: usize) -> usize {
        (current_turn + 1) % num_seats
    }
}