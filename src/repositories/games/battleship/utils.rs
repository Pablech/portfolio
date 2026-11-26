use crate::repositories::games::battleship::AgentSea;

pub fn ship_cells(anchor: usize, size: u8, vertical: bool) -> Vec<usize> {
    (0..size as usize)
        .map(|i| if vertical { anchor + i * 10 } else { anchor + i })
        .collect()
}

pub fn area_clear(sea: &AgentSea, cells: &[usize]) -> bool {
    for &c in cells {
        if sea.ships[c] != 0 { return false; }

        let (row, col) = (c / 10, c % 10);
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                let (nr, nc) = (row as i32 + dr, col as i32 + dc);
                if !(0..10 as i32).contains(&nr) || !(0..10 as i32).contains(&nc) { continue; }
                let n = (nr as usize) * 10 + nc as usize;
                if sea.ships[n] != 0 && !cells.contains(&n) { return false; }
            }
        }
    }
    true
}