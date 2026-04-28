pub enum Era {
    NineteenEightyFour,
}

impl Era {
    pub fn name(&self) -> &'static str {
        match self {
            Era::NineteenEightyFour => "1984",
        }
    }
}
