use crate::window::Window;
use std::fmt;

pub mod lua;

// global functions
pub const ALERT: &'static str = "alert";
pub const FIND_WIDGET: &'static str = "find_widget";
pub const SUBMIT_FORM: &'static str = "submit_form";
pub const FETCH: &'static str = "fetch";

// global vars
pub const WINDOW: &'static str = "window";

// widget functions
pub const CONNECT: &'static str = "connect";
pub const GET_PROPERTY: &'static str = "get_property";
pub const SET_PROPERTY: &'static str = "set_property";
pub const GET_TEXT: &'static str = "get_text";
pub const SET_SENSITIVE: &'static str = "set_sensitive";
pub const SET_LABEL: &'static str = "set_label";
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

pub struct Script {
    pub lang: Lang,
    pub text: String,
}

impl Script {
    pub fn new(lang: Lang, text: String) -> Script {
        Script { lang, text }
    }

    pub fn execute(&self, window: &Window) {
        match self.lang {
            Lang::Lua => {
                println!("Executing Lua: {}", &self.text);
                if let Err(err) = window.state.borrow().globals.lua.load(&self.text).exec() {
                    println!("Lua script execution error: {}", err);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    UnsupportedOperation,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unsupported operation")
    }
}

impl std::error::Error for Error {}
