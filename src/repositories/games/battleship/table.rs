use std::ops::Index;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::repositories::games::{Agent, GameError, GameLogic, battleship::utils::{area_clear, ship_cells},};

pub const GRID: usize = 10;
pub const CELLS: usize = GRID * GRID;

pub const FLEET: [(u8, u8, &str); 6] = [
    (1, 5, "Porta-aviões"),
    (2, 4, "Navio-tanque"),
    (3, 3, "Contratorpedeiro A"),
    (4, 3, "Contratorpedeiro B"),
    (5, 2, "Submarino A"),
    (6, 2, "Submarino B"),
];

const LAST: &str = "red";

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct AgentSea {
    #[serde_as(as = "[_; CELLS]")]
    pub ships: [u8; CELLS],
    #[serde_as(as = "[_; CELLS]")]
    pub shots: [Option<bool>; CELLS],
    pub hits: [u8; 6],
    pub last_move: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BattleShipTable {
    pub tables: [AgentSea; 2],
}

impl Index<usize> for BattleShipTable {
    type Output = AgentSea;
    fn index(&self, index: usize) -> &Self::Output {
        &self.tables[index]
    }
}

impl Default for AgentSea {
    fn default() -> Self {
        Self { ships: [0; CELLS], shots: [None; CELLS], hits: [0; 6], last_move: None }
    }
}

impl Default for BattleShipTable {
    fn default() -> Self {
        Self { tables: [AgentSea::default(); 2], }
    }
}

impl AgentSea {
    pub fn remaining(&self) -> u8 {
        FLEET.iter().enumerate()
            .filter(|&(i, &(_, size, _))| self.hits[i] < size)
            .count() as u8
    }

    pub fn random<R: rand::Rng>(rng: &mut R) -> Self {
        loop {
            let mut sea = Self::default();
            let mut ok = true;
            for &(id, size, _) in &FLEET {
                let mut cands = Vec::new();
                for anchor in 0..CELLS {
                    for vertical in [false, true] {
                        let (row, col) = (anchor / GRID, anchor % GRID);
                        let fits = if vertical {
                            row + size as usize <= GRID
                        } else {
                            col + size as usize <= GRID
                        };
                        if fits {
                            let cells = ship_cells(anchor, size, vertical);
                            if area_clear(&sea, &cells) {
                                cands.push((anchor, vertical));
                            }
                        }
                    }
                }
                match cands.choose(rng) {
                    Some(&(anchor, vertical)) => {
                        for c in ship_cells(anchor, size, vertical) {
                            sea.ships[c] = id;
                        }
                    }
                    None => { ok = false; break; }
                }
            }
            if ok {
                return sea;
            }
        }
    }
}

impl BattleShipTable {
    pub fn started(&self) -> bool {
        self.tables.iter().all(|s| s.ships.iter().any(|&id| id!= 0))
    }

    pub fn deal_host(&mut self, rng: &mut impl rand::Rng) {
        self.tables[0] = AgentSea::random(rng);
    }

    pub fn deal_guest(&mut self, rng: &mut impl rand::Rng) {
        loop {
            let sea = AgentSea::random(rng);
            if sea.ships != self.tables[0].ships {
                self.tables[1] = sea;
                break;
            }
        }
    }

    fn my_cell(&self, seat: usize, c: usize) -> char {
        let me = &self.tables[seat];
        match me.shots[c] {
            Some(true)  => 'X',
            Some(false) => ' ',
            None if me.ships[c] != 0 => '+',
            _ => '~',
        }
    }

    fn foe_cell(&self, seat: usize, c: usize) -> char {
        let foe = &self.tables[1 - seat];
        match foe.shots[c] {
            None => '~',
            Some(false) => ' ',
            Some(true) => {
                let id = foe.ships[c];
                let (_, size, _) = FLEET[(id - 1) as usize];
                if foe.hits[(id - 1) as usize] >= size { '†' } else { 'X' }
            }
        }
    }

    fn render_hot(base: char, hot: bool) -> String {
        if hot {
            format!("<span style=\"color:{LAST};\">{base}</span>")
        } else {
            base.to_string()
        }
    }

    fn my_hot(&self, seat: usize, c: usize, last: Option<usize>) -> bool {
        let me = &self.tables[seat];
        if me.shots[c] == Some(true) {
            let id = me.ships[c];
            let (_, size, _) = FLEET[(id - 1) as usize];
            if me.hits[(id - 1) as usize] >= size { return true; }
        }
        last == Some(c)
    }

    fn foe_hot(&self, seat: usize, c: usize, last: Option<usize>) -> bool {
        self.foe_cell(seat, c) == '†' || last == Some(c)
    }

    fn coord(c: usize) -> String {
        format!("{}{}", (b'A' + (c / GRID) as u8) as char, c % GRID)
    }

    fn parse_coord(raw: &str) -> Result<usize, GameError> {
        let s = raw.trim().to_ascii_uppercase();
        let mut it = s.chars();
        let (row, col) = match (it.next(), it.next()) {
            (Some(r), Some(c)) if it.next().is_none() => (r, c),
            _ => return Err(GameError::ErrorMove(
                "Formato inválido. Use letra (A-J) + número (0-9), ex: B3".into(),
            )),
        };

        let row_idx = match row {
            'A'..='J' => (row as u8 - b'A') as usize,
            _ => return Err(GameError::ErrorMove(
                "Linha inválida: use A a J".into(),
            )),
        };
        let col_idx = match col.to_digit(10) {
            Some(d) if d < 10 => d as usize,
            _ => return Err(GameError::ErrorMove(
                "Coluna inválida: use 0 a 9".into(),
            )),
        };

        Ok(row_idx * GRID + col_idx)
    }
}

impl GameLogic for BattleShipTable {
    const SEATS: usize = 2;

    fn new() -> Self where Self: Sized {
        Self::default()
    }

    fn legal_moves(&self) -> Vec<usize> {
        (0..CELLS).filter(|&c| {
            self.tables[0].shots[c].is_none() || self.tables[1].shots[c].is_none()
        }).collect()
    }
    
    fn place(&mut self, mv: usize, player: u8) -> Result<(), GameError> {
        if mv >= CELLS {
            return Err(GameError::ErrorMove("Cell deve ser menor que 100".to_string()));
        }

        let seat = player as usize - 1;
        let foe = 1 - seat;

        if !self.started() {
            return Err(GameError::ErrorMove("partida ainda não começou — aguardando oponente".into()));
        }

        if self.tables[foe].shots[mv].is_some() {
            return Err(GameError::ErrorMove(format!("já atirou nesta célula: {}", Self::move_label(mv))));
        }

        let hit = self.tables[foe].ships[mv] != 0;
        self.tables[foe].shots[mv] = Some(hit);
        self.tables[seat].last_move = Some(mv);

        if hit {
            let id = self.tables[foe].ships[mv];
            self.tables[foe].hits[(id - 1) as usize] += 1;
        }

        Ok(())
    }

    fn check_win(&self, fig: u8) -> bool {
        if !self.started() { return false; }
        let foe = 1 - (fig as usize - 1);
        self.tables[foe].remaining() == 0
    }

    fn check_draw(&self) -> bool {
        false
    }

    fn output(&self, players: &[Agent], seat: &Option<usize>) -> String {
        let _ = players;
        let Some(s) = seat else { return String::new(); };
        let my_last = self.tables[*s].last_move;
        let foe_last = self.tables[1 - s].last_move;

        let mut out = String::new();
        out.push_str("Seu mar                    Mar do inimigo\n");
        out.push_str("   0 1 2 3 4 5 6 7 8 9        0 1 2 3 4 5 6 7 8 9\n");

        for row in 0..GRID {
            let label = (b'A' + row as u8) as char;
            let mut mine = format!("{label} ");
            let mut theirs = format!("{label} ");

            for col in  0..GRID {
                let c = row * GRID + col;
                mine.push(' ');
                mine.push_str(&Self::render_hot(self.my_cell(*s, c), self.my_hot(*s, c, foe_last)));
                theirs.push(' ');
                theirs.push_str(&Self::render_hot(self.foe_cell(*s, c), self.foe_hot(*s, c, my_last)));
            }
            out.push_str(&mine);
            out.push_str("     ");
            out.push_str(&theirs);
            out.push('\n');
        }

        out.push('\n');

        if let Some(m) = my_last {
            let result = if self.tables[1 - s].shots[m] == Some(true) { "acertou" } else { "água" };
            out.push_str(&format!("Seu último tiro: {} ({})\n", Self::coord(m), result));
        }
        if let Some(f) = foe_last {
            out.push_str(&format!("Último tiro do inimigo: {}\n", Self::coord(f)));
        }
        out.push_str(&format!(
            "Frota — você: {}, inimigo: {}\n",
            self.tables[*s].remaining(),
            self.tables[1 - s].remaining()
        ));
        out
    }

    fn shared_output(&self) -> bool {
        false
    }

    fn parse_input(input: &str) -> Result<usize, GameError> {
        Self::parse_coord(input)
    }

    fn move_label(mv: usize) -> String {
        format!("{}{}", (b'A' + (mv / GRID) as u8) as char, mv % GRID)
    }

    fn on_host_seated(&mut self) {
        self.deal_host(&mut rand::thread_rng());
    }

    fn on_table_full(&mut self) {
        self.deal_guest(&mut rand::thread_rng());
    }
}
