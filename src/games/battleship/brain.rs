use rand::seq::SliceRandom;

use crate::games::{CpuBrain, SmartishBrain, 
    battleship::{AgentSea, BattleShipTable, CELLS, FLEET, GRID, utils::ship_cells,}};

const ORTHO: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

fn sunk_halo(foe: &AgentSea, c: usize) -> bool {
    let (row, col) = (c / GRID, c % GRID);
    for dr in -1i32..=1 {
        for dc in -1i32..=1 {
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if !(0..GRID as i32).contains(&nr) || !(0..GRID as i32).contains(&nc) {
                continue;
            }
            let n = nr as usize * GRID + nc as usize;
            let id = foe.ships[n];
            if id != 0 && ship_sunk(foe, id) {
                return true;
            }
        }
    }
    false
}

fn ship_sunk(foe: &AgentSea, id: u8) -> bool {
    let (_, size, _) = FLEET[(id - 1) as usize];
    foe.hits[(id - 1) as usize] >= size
}

fn pending_hits(foe: &AgentSea) -> Vec<usize> {
    (0..CELLS)
        .filter(|&c| foe.shots[c] == Some(true) && !ship_sunk(foe, foe.ships[c]))
        .collect()
}

fn density(foe: &AgentSea) -> [u16; CELLS] {
    let mut map = [0u16;CELLS];
    for &(id, size, _) in &FLEET {
        if ship_sunk(foe, id) {
            continue;
        }
        for anchor in 0..CELLS {
            for vertical in [false, true] {
                let (row, col) = (anchor / GRID, anchor % GRID);
                let s = size as usize;
                if vertical && row + s > GRID { continue; }
                if !vertical && col + s > GRID { continue; }
                let cells = ship_cells(anchor, size, vertical);
                if cells.iter().any(|&c| foe.shots[c] == Some(false) || sunk_halo(foe, c)) { continue; }
                for &c in &cells {
                    if foe.shots[c].is_none() { map[c] += 1 }
                }
            }
        }
    }
    map
}

fn adjacent(a: usize, b: usize) -> bool {
    let (dr, dc) = (a / GRID, a % GRID);
    let (br, bc) = (b / GRID, b % GRID);
    dr.abs_diff(br) + dc.abs_diff(bc) == 1
}

fn min_alive_size(foe: &AgentSea) -> usize {
    FLEET.iter()
        .filter(|&&(id, _, _)| !ship_sunk(foe, id))
        .map(|&(_, size, _)| size as usize)
        .min().unwrap_or(2)
}

fn targets(foe: &AgentSea, pending: &[usize]) -> Vec<(usize, u32)> {
    let has_row_pair = pending.iter().enumerate()
        .any(|(i, &a)| pending[i + 1..].iter().any(|&b| a / GRID == b / GRID));
    let has_col_pair = pending.iter().enumerate()
        .any(|(i, &a)| pending[i + 1..].iter().any(|&b| a % GRID == b % GRID));

    let mut cands: Vec<(usize, u32)> = Vec::new();
    for &h in pending {
        let (row, col) = (h / GRID, h % GRID);
        for (dr, dc) in ORTHO {
            let (nr, nc) = (row as i32 + dr, col as i32 + dc);
            if !(0..GRID as i32).contains(&nr) || !(0..GRID as i32).contains(&nc) { continue; }
            let n = nr as usize * GRID + nc as usize;
            if foe.shots[n].is_some() || cands.iter().any(|&(c, _)| c == n) { continue; }
            let mut score = 1u32;
            score += pending.iter().filter(|&&p| p != h && adjacent(n, p)).count() as u32;

            if has_row_pair && n / GRID == row { score += 2; }
            if has_col_pair && n % GRID == col { score += 2; }
            cands.push((n, score));
        }
    }
    cands
}

impl CpuBrain<BattleShipTable> for SmartishBrain {
    fn choose(&self, table: &BattleShipTable, legal: &[usize], me: u8) -> usize {
        let seat = (me - 1) as usize;
        let foe = &table[1 - seat];

        let map = density(foe);
        let pending = pending_hits(foe);

        if !pending.is_empty() {
            let cands = targets(foe, &pending);
            if let Some((best_key, _)) = cands.iter().map(|&(c, score)| ((score, map[c] as u32), c)).max_by_key(|&(k, _)| k) {
                let top: Vec<usize> = cands
                    .into_iter()
                    .filter(|&(c, score)| (score, map[c] as u32) == best_key)
                    .map(|(c, _)| c)
                    .collect();
                if let Some(&mv) = top.choose(&mut rand::thread_rng()) {
                    return mv;
                }
            }
        }

        let s_min = min_alive_size(foe);
        let pool: Vec<(usize, u32)> = (0..CELLS)
            .filter(|&c| foe.shots[c].is_none())
            .map(|c| {
                let parity = if (c / GRID + c % GRID).is_multiple_of(s_min) { 3 } else { 1 };
                (c, map[c] as u32 * parity)
            })
            .collect();

        if !pool.is_empty() {
            if let Ok(chosen) = pool.choose_weighted(&mut rand::thread_rng(), |&(_, w)| w + 1) {
                return chosen.0;
            }
        }

        legal
            .iter()
            .copied()
            .find(|&c| foe.shots[c].is_none())
            .or_else(|| legal.first().copied())
            .unwrap_or(0)
    }
}