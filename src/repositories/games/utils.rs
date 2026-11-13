// use rand;

pub fn generate_token() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

// pub fn dice(n: usize) -> usize {
//     rand::thread_rng().gen_range(0..n)
// }