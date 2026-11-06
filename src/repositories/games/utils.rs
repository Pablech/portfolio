pub fn generate_token() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}