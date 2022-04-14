use crate::window::Window;

pub mod lua;

// global functions
pub const ALERT: &'static str = "alert";
pub const FIND_WIDGET: &'static str = "find_widget";

// global vars
pub const WINDOW: &'static str = "window";

// widget functions
pub const CONNECT: &'static str = "connect";
pub const SET_SENSITIVE: &'static str = "set_sensitive";
pub const ADD_CSS_CLASS: &'static str = "add_css_class";
pub const REMOVE_CSS_CLASS: &'static str = "remove_css_class";
pub const SET_CSS_CLASSES: &'static str = "set_css_classes";

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
                println!("Executing Lua: {}", &self.text);
                if let Err(err) = window.state.borrow().globals.lua.load(&self.text).exec() {
                    println!("Lua script execution error: {}", err);
                }
            },
        }
    }
}
