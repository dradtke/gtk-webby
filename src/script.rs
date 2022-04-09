#[derive(Copy, Clone, Debug)]
pub enum Lang {
    Lua,
}

impl Lang {
    pub fn from(str: &str) -> Option<Lang> {
        match str {
            "lua" => Some(Lang::Lua),
            _ => None,
        }
    }
}
