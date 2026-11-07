use rand::seq::SliceRandom;

use crate::repositories::games::structs::{Cpu, Human};

const CPU_FIGS: &[char] = &['@', '&', '%', 'C', '?', '$', '#'];

impl Human {
    pub fn new(name: String, fig: char) -> Self {
        Self { name: name, fig: fig }
    }
}

impl Cpu {
    pub fn new(figs: Vec<char>) ->Self {
        let mut candidates: Vec<char> = CPU_FIGS
            .iter()
            .copied()
            .filter(|f| !figs.contains(f))
            .collect();

        let fig = candidates
            .choose_mut(&mut rand::thread_rng())
            .copied()
            .unwrap_or('?');
        
        Self { name: "Cpu.ai".into(), fig: fig, players_fig: figs }
    }
}
