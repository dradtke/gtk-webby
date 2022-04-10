use crate::window::Window;

pub mod lua;

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

pub struct Script{
    pub lang: Lang,
    pub text: String,
}

impl Script {
    pub fn new(lang: Lang, text: String) -> Script {
        Script{lang, text}
    }

    pub fn execute(&self, window: &Window) {
        match self.lang {
            Lang::Lua => {
                if let Err(err) = window.state.borrow().lua.load(&self.text).exec() {
                    println!("script execution error: lua: {}", err);
                }
            },
        }
    }
}
