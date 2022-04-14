use gtk::prelude::*;
use mlua::prelude::*;

use gtk::glib;
use glib::signal::SignalHandlerId;
use std::rc::Rc;

pub fn init(window: Rc<crate::window::Window>) {
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
            lua_globals.set(super::ALERT, alert)?;
        }
        {
            let window = window.clone();
            let find_widget = lua.create_function(move |_, id: String| {
                match window.state.borrow().builder.object::<gtk::Widget>(&id) {
                    // TODO: need to figure out how to drop this widget after it's no longer visible
                    Some(widget) => Ok(Some(Widget::new(globals, widget))),
                    None => {
                        println!("No widget found with id: {}", &id);
                        Ok(None)
                    },
                }
            })?;
            lua_globals.set(super::FIND_WIDGET, find_widget)?;
        }
        lua_globals.set(super::WINDOW, Window{globals, window: window.clone()})?;
        Ok(())
    };

    if let Err(err) = r#do() {
        println!("Failed to register lua globals: {}", err);
    }
}

fn glib_to_lua(value: &glib::Value) -> Option<LuaValue> {
    use glib::types::Type;
    match value.type_() {
        Type::INVALID | Type::UNIT => Some(LuaValue::Nil), // not sure if it's possible to initialize a unit value...
        Type::BOOL => Some(LuaValue::Boolean(value.get().unwrap())),
        // TODO: add more types
        t => {
            println!("Unimplemented glib->Lua conversion: {:?}", t);
            None
        },
    }
}

fn lua_to_glib(value: &LuaValue) -> Option<glib::Value> {
    use LuaValue::*;
    match value {
        Nil => None,
        Boolean(v) => Some(v.to_value()),
        t => {
            println!("Unimplemented Lua->glib conversion: {:?}", t);
            None
        },
    }
}

struct Widget {
    globals: &'static crate::Globals,
    widget: gtk::Widget,
    // ???: Do Lua registry keys need to be manually deregistered, or signals manually
    // disconnected, in order to prevent memory leaks or weird behavior?
    registry_keys: Vec<Rc<LuaRegistryKey>>,
    signal_ids: Vec<(gtk::Widget, SignalHandlerId)>,
}

impl Widget {
    fn new(globals: &'static crate::Globals, widget: gtk::Widget) -> Self {
        Self{
            globals,
            widget,
            signal_ids: Vec::new(),
            registry_keys: Vec::new(),
        }
    }
}

impl LuaUserData for Widget {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(super::CONNECT, |_, this, (signal, after, callback): (String, bool, LuaFunction)| {
            let lua = &this.globals.lua;
            let callback_key = Rc::new(lua.create_registry_value(callback).expect("Failed to create Lua registry value"));
            let weak_callback_ref = Rc::downgrade(&callback_key);
            this.registry_keys.push(callback_key);

            let signal_id = this.widget.connect_local(&signal, after, move |values| {
                let lua_values = match values.iter().map(glib_to_lua).collect::<Option<Vec<LuaValue>>>() {
                    Some(lua_values) => lua_values,
                    None => {
                        println!("Failed to convert one or more glib values to Lua");
                        return None;
                    },
                };

                let key = match weak_callback_ref.upgrade() {
                    Some(key) => key,
                    None => {
                        println!("Callback missing from Lua registry");
                        return None;
                    },
                };

                let f: LuaFunction = lua.registry_value(&key).unwrap();
                let retvals = match f.call::<_, LuaMultiValue>(lua_values) {
                    Ok(retval) => retval,
                    Err(err) => {
                        println!("Error calling Lua callback: {:?}", err);
                        return None;
                    },
                }.into_vec();

                match retvals.len() {
                    0 => None,
                    1 => lua_to_glib(&retvals[0]),
                    n => {
                        println!("Cannot return {} values in callback", n);
                        None
                    },
                }
            });

            this.signal_ids.push((this.widget.clone(), signal_id));
            Ok(())
        });

        methods.add_method(super::SET_SENSITIVE, |_, this, sensitive: bool| {
            this.widget.set_sensitive(sensitive);
            Ok(())
        });

        methods.add_method(super::ADD_CSS_CLASS, |_, this, css_class: String| {
            this.widget.add_css_class(&css_class);
            Ok(())
        });

        methods.add_method(super::REMOVE_CSS_CLASS, |_, this, css_class: String| {
            this.widget.remove_css_class(&css_class);
            Ok(())
        });

        methods.add_method(super::SET_CSS_CLASSES, |_, this, classes: Vec<String>| {
            let v: Vec<&str> = classes.iter().map(String::as_ref).collect();
            this.widget.set_css_classes(&v);
            Ok(())
        });
    }
}

struct Window {
    globals: &'static crate::Globals,
    window: Rc<crate::window::Window>,
}

impl LuaUserData for Window {
    // TODO: implement set_title()
}

#[cfg(test)]
mod test {
    use super::*;
    use glib::value::{Value, ToValue};
    use glib::types::Type;

    #[test]
    pub fn test_glib_value_to_lua() {
        // TODO: finish adding types, and figure out if it's possible to do null
        assert_eq!(glib_to_lua(&true.to_value()), Some(LuaValue::Boolean(true)));
        assert_eq!(glib_to_lua(&false.to_value()), Some(LuaValue::Boolean(false)));
    }
}
