use gtk::prelude::*;
use mlua::prelude::*;

use crate::window::Window;
use gtk::glib;
use glib::closure::RustClosure;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use std::mem::{transmute, ManuallyDrop};

pub fn init(window: Rc<Window>) {
    let globals = window.state.borrow().globals;
    let lua = &globals.lua;

    let r#do = move || -> LuaResult<()> {
        let lua_globals = lua.globals();
        {
            // Lotta cloning here, not sure how to clean it up.
            let window = window.clone();
            let alert = lua.create_function(move |_, text: String| {
                window.clone().alert(&text);
                Ok(())
            })?;
            lua_globals.set("alert", alert)?;
        }
        {
            let window = window.clone();
            let find_widget = lua.create_function(move |_, id: String| {
                match window.state.borrow().builder.object::<gtk::Widget>(&id) {
                    Some(widget) => Ok(Some(Widget(globals, widget))),
                    None => {
                        println!("No widget found with id: {}", &id);
                        Ok(None)
                    },
                }
            })?;
            lua_globals.set("find_widget", find_widget)?;
        }
        Ok(())
    };

    if let Err(err) = r#do() {
        println!("Failed to register lua globals: {}", err);
    }
}

struct Widget(&'static crate::Globals, gtk::Widget);

impl LuaUserData for Widget {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("connect", |_, this, (signal, callback): (String, LuaFunction)| {
            let lua = &this.0.lua;
            let callback_key = lua.create_registry_value(callback).unwrap(); // TODO: handle more gracefully
            this.1.connect_local(&signal, false, move |_values| {
                let f: LuaFunction = lua.registry_value(&callback_key).unwrap();
                f.call::<(), ()>(());
                None
            });
            Ok(())
        });
    }
}
