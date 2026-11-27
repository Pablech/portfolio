use rand::prelude::IteratorRandom;

const RANDOM_FIGS: &[char] = &['@', '&', '%', 'C', '?', '$', '#', 'O',];

pub fn generate_token() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn choice_fig(taken: &[char]) -> char {
    RANDOM_FIGS
        .iter()
        .copied()
        .filter(|f| !taken.contains(f))
        .choose(&mut rand::thread_rng())
        .expect("figuras esgotadas — amplia RANDOM_FIGS")
}

pub fn resolve_name(requested: Option<String>, default_name: &str, 
        seat: usize, taken_names: &[String] ) -> String {
    match requested {
        Some(n) if taken_names.contains(&n) => format!("{} {}", n, seat + 1),
        Some(n) => n,
        None => default_name.to_owned(),
    }
}