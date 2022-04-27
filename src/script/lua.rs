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
                    Some(widget) => Ok(Some(Widget::new(lua, widget))),
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

fn glib_to_lua<'v>(lua: &'static Lua, value: &'v glib::Value) -> Option<LuaValue<'v>> {
    use glib::types::Type;
    let mut current_type = Some(value.type_());
    while let Some(t) = current_type {
        match t {
            Type::INVALID | Type::UNIT => return Some(LuaValue::Nil), // not sure if it's possible to initialize a unit value...
            Type::BOOL => return Some(LuaValue::Boolean(value.get().unwrap())),
            _ => (),
        }
        if t == gtk::Widget::static_type() {
            let transformed_value = match value.transform_with_type(t) {
                Ok(v) => v,
                Err(err) => {
                    println!("failed to transform value '{:?}' into type '{:?}': {}", value, t, err);
                    return None
                },
            };
            let widget: gtk::Widget = match transformed_value.get() {
                Ok(widget) => widget,
                Err(err) => {
                    println!("failed to extract value as widget: {}", err);
                    return None
                },
            };

            // Okay, so putting the userdata into a table works, but using it directly in a
            // function call doesn't...
            {
                lua.globals().set("special_widget_value", Widget::new(lua, widget.clone()).to_lua(lua).unwrap()).unwrap();
            }
            let lua_widget = match Widget::new(lua, widget).to_lua(lua) {
                Ok(lua_widget) => lua_widget,
                Err(err) => {
                    println!("failed to convert widget into Lua value: {}", err);
                    return None
                },
            };
            return Some(lua_widget);
        }
        current_type = t.parent();
    }
    None
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
    lua: &'static Lua,
    widget: gtk::Widget,
    // ???: Do Lua registry keys need to be manually deregistered, or signals manually
    // disconnected, in order to prevent memory leaks or weird behavior?
    registry_keys: Vec<Rc<LuaRegistryKey>>,
    signal_ids: Vec<(gtk::Widget, SignalHandlerId)>,
}

impl Widget {
    fn new(lua: &'static Lua, widget: gtk::Widget) -> Self {
        Self{
            lua,
            widget,
            signal_ids: Vec::new(),
            registry_keys: Vec::new(),
        }
    }
}

impl LuaUserData for Widget {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(super::CONNECT, |_, this, (signal, after, callback): (String, bool, LuaFunction)| {
            let lua: &'static Lua = this.lua;

            let callback_key = Rc::new(lua.create_registry_value(callback).expect("Failed to create Lua registry value"));
            let weak_callback_ref = Rc::downgrade(&callback_key);
            this.registry_keys.push(callback_key);

            let signal_id = this.widget.connect_local(&signal, after, move |values| {
                /*
                let lua_values = match values.iter().map(|v| glib_to_lua(lua, v)).collect::<Option<Vec<LuaValue>>>() {
                    Some(lua_values) => lua_values,
                    None => {
                        println!("Failed to convert one or more glib values to Lua");
                        return None;
                    },
                };
                */

                let key = match weak_callback_ref.upgrade() {
                    Some(key) => key,
                    None => {
                        println!("Callback missing from Lua registry");
                        return None;
                    },
                };

                // NOTE: This is very hacky, but when passing the widget reference into the
                // callback arguments directly, it doesn't seem to have any methods registered.
                // TODO: try to cast values[0] to a widget
                if values.len() == 1 {
                    if let Ok(widget_value) = values[0].transform_with_type(gtk::Widget::static_type()) {
                        lua.globals().set("this", Widget::new(lua, widget_value.get().unwrap()));
                    }
                }

                let f: LuaFunction = lua.registry_value(&key).unwrap();
                let retvals = match f.call::<_, LuaMultiValue>(/*lua_values*/()) {
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

        methods.add_method(super::SET_LABEL, |_, this, label: String| {
            if let Ok(button) = this.widget.clone().downcast::<gtk::Button>() {
                button.set_label(&label);
            }
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
        let lua = Box::leak(Box::new(Lua::new()));
        assert_eq!(glib_to_lua(lua, &true.to_value()), Some(LuaValue::Boolean(true)));
        assert_eq!(glib_to_lua(lua, &false.to_value()), Some(LuaValue::Boolean(false)));
    }
}
