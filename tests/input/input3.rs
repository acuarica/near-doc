#[near_bindgen]
impl S {
    pub fn get(&self) -> i32 {
        self.f
    }
}

#[derive(Serialize)]
struct T(u32, bool);

#[derive(Serialize)]
struct U(AccountId);
