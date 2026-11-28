use rand::seq::SliceRandom;

use crate::games::{CpuBrain, SmartishBrain, connect4::Conn4Table};


impl CpuBrain<Conn4Table> for SmartishBrain {
    fn choose(&self, table: &Conn4Table, legal: &[usize], me: u8) -> usize {
        let opp = if me == 1 { 2 } else { 1 };

        for &mv in legal {
            if table.is_win_with(mv, me) {
                return mv;
            }
        }

        for &mv in legal {
            if table.is_win_with(mv, opp) {
                return mv;
            }
        }

        *legal.choose(&mut rand::thread_rng())
            .expect("sem movimentos legais")
    }
}