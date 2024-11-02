#[derive(Clone)]
pub struct FurUser {
    pub email: String,
    pub password_hash: String,
    pub salt: Vec<u8>,
    pub server: String,
}

impl Default for FurUser {
    fn default() -> Self {
        FurUser {
            email: String::new(),
            password_hash: String::new(),
            salt: Vec::new(),
            server: String::new(),
        }
    }
}
