use gtk::prelude::*;
use mlua::prelude::*;

use glib::signal::SignalHandlerId;
use glib::{Continue, MainContext, PRIORITY_DEFAULT};
use gtk::glib;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub fn init(window: Rc<crate::window::Window>) {
    let globals = window.state.borrow().globals;
    let lua = &globals.lua;

    let r#do = move || -> LuaResult<()> {
        for (name, function) in global_functions(lua, window.clone())? {
            lua.globals().set(name, function)?;
        }
        lua.globals().set(
            super::WINDOW,
            Window {
                globals,
                window: window.clone(),
            },
        )?;
        Ok(())
    };

    if let Err(err) = r#do() {
        println!("Failed to register lua globals: {}", err);
    }
}

fn global_functions(
    lua: &'static Lua,
    window: Rc<crate::window::Window>,
) -> LuaResult<HashMap<&'static str, LuaFunction>> {
    let mut functions = HashMap::new();

    {
        let window = window.clone();
        functions.insert(
            super::ALERT,
            lua.create_function(move |_, text: String| {
                window.clone().alert(&text);
                Ok(())
            })?,
        );
    }

    {
        let window = window.clone();
        functions.insert(
            super::FIND_WIDGET,
            lua.create_function(move |_, id: String| {
                match window.state.borrow().builder.object::<gtk::Widget>(&id) {
                    // TODO: need to figure out how to drop this widget after it's no longer visible
                    Some(widget) => Ok(Some(Widget::new(lua, widget))),
                    None => {
                        println!("No widget found with id: {}", &id);
                        Ok(None)
                    }
                }
            })?,
        );
    }

    {
        let window = window.clone();
        functions.insert(
            super::SUBMIT_FORM,
            lua.create_function(
                move |_, (method, action, values): (String, String, LuaTable)| {
                    let method = match reqwest::Method::from_bytes(method.as_bytes()) {
                        Ok(method) => method,
                        Err(err) => {
                            return Err(LuaError::ExternalError(Arc::new(err)));
                        }
                    };

                    let mut form_values = HashMap::new();
                    // TODO: automatically convert other types, like boolean?
                    for pair in values.pairs::<String, String>() {
                        let (key, value) = pair?;
                        form_values.insert(key, value);
                    }

                    let http_client = &window.state.borrow().http_client;
                    let response = match http_client
                        .request(
                            method,
                            crate::util::absolutize_url(&window.state.borrow().location, &action),
                        )
                        .form(&form_values)
                        .send()
                    {
                        Ok(response) => response,
                        Err(err) => {
                            return Err(LuaError::ExternalError(Arc::new(err)));
                        }
                    };

                    if response.status().is_success() {
                        window.clone().reload();
                    } else if response.status().is_redirection() {
                        println!("TODO: Need to redirect, probably to {}", response.url());
                    }
                    Ok(())
                },
            )?,
        );
    }

    {
        let window = window.clone();
        functions.insert(
            super::FETCH,
            lua.create_function(
                move |_, (method, url, callback): (String, String, LuaFunction)| {
                    if !url.contains("://") {
                        if let Err(err) = callback.call::<_, ()>((
                            format!("URL is missing protocol: {}", url),
                            LuaValue::Nil,
                        )) {
                            println!("Failed to invoke fetch callback: {}", err);
                        }
                        return Ok(());
                    }

                    let method = match reqwest::Method::from_bytes(method.as_bytes()) {
                        Ok(method) => method,
                        Err(err) => {
                            return Err(LuaError::ExternalError(Arc::new(err)));
                        }
                    };

                    let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

                    let request = window.state.borrow().http_client.request(method, url);
                    std::thread::spawn(move || {
                        let response_result = request.send();
                        if let Err(err) = sender.send(response_result) {
                            println!("fetch: Failed to send response on channel: {}", err);
                        }
                    });

                    let callback_key = lua
                        .create_registry_value(callback)
                        .expect("Failed to create Lua registry value");
                    receiver.attach(None, move |response_result| {
                        let f: LuaFunction = lua.registry_value(&callback_key).unwrap();
                        match response_result {
                            Ok(response) => {
                                if let Err(err) =
                                    f.call::<_, ()>((LuaValue::Nil, Response::new(response)))
                                {
                                    println!("Failed to invoke fetch callback: {}", err);
                                }
                            }
                            Err(err) => {
                                if let Err(err) = f.call::<_, ()>((
                                    LuaError::ExternalError(Arc::new(err)),
                                    LuaValue::Nil,
                                )) {
                                    println!("Failed to invoke fetch callback: {}", err);
                                }
                            }
                        }
                        // lua.remove_registry_value(callback_key);
                        Continue(false)
                    });
                    Ok(())
                },
            )?,
        );
    }

    Ok(functions)
}

#[allow(dead_code)]
fn glib_to_lua(lua: &'static Lua, value: glib::Value) -> Option<LuaValue> {
    println!("glib_to_lua: converting {:?}", &value);
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
                    println!(
                        "failed to transform value '{:?}' into type '{:?}': {}",
                        value, t, err
                    );
                    return None;
                }
            };
            let widget: gtk::Widget = match transformed_value.get() {
                Ok(widget) => widget,
                Err(err) => {
                    println!("failed to extract value as widget: {}", err);
                    return None;
                }
            };

            // Okay, so putting the userdata into a table works, but using it directly in a
            // function call doesn't...
            {
                lua.globals()
                    .set(
                        "special_widget_value",
                        Widget::new(lua, widget.clone()).to_lua(lua).unwrap(),
                    )
                    .unwrap();
            }
            let lua_widget = match Widget::new(lua, widget).to_lua(lua) {
                Ok(lua_widget) => lua_widget,
                Err(err) => {
                    println!("failed to convert widget into Lua value: {}", err);
                    return None;
                }
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
        }
    }
}

struct Widget {
    lua: &'static Lua,
    widget: gtk::Widget,
    signal_ids: Vec<(gtk::Widget, SignalHandlerId)>,
}

impl Widget {
    fn new(lua: &'static Lua, widget: gtk::Widget) -> Self {
        Self {
            lua,
            widget,
            signal_ids: Vec::new(),
        }
    }
}

impl LuaUserData for Widget {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            super::CONNECT,
            |_, this, (signal, after, callback): (String, bool, LuaFunction)| {
                let lua: &'static Lua = this.lua;

                let callback_key = lua
                    .create_registry_value(callback)
                    .expect("Failed to create Lua registry value");
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

                    // NOTE: This is very hacky, but when passing the widget reference into the
                    // callback arguments directly, it doesn't seem to have any methods registered.
                    // TODO: try to cast values[0] to a widget
                    if values.len() > 0 {
                        if let Ok(widget_value) =
                            values[0].transform_with_type(gtk::Widget::static_type())
                        {
                            if let Err(err) = lua
                                .globals()
                                .set("this", Widget::new(lua, widget_value.get().unwrap()))
                            {
                                println!("Failed to set 'this' value before callback: {}", err);
                            }
                        }
                    }

                    let f: LuaFunction = lua.registry_value(&callback_key).unwrap();
                    let retvals = match f.call::<_, LuaMultiValue>(/*lua_values*/ ()) {
                        Ok(retval) => retval,
                        Err(err) => {
                            println!("Error calling Lua callback: {:?}", err);
                            return None;
                        }
                    }
                    .into_vec();

                    match retvals.len() {
                        0 => None,
                        1 => lua_to_glib(&retvals[0]),
                        n => {
                            println!("Cannot return {} values in callback", n);
                            None
                        }
                    }
                });

                this.signal_ids.push((this.widget.clone(), signal_id));
                Ok(())
            },
        );

        methods.add_method(
            super::GET_PROPERTY,
            |_, this, property_name: String| match this.widget.find_property(&property_name) {
                Some(prop) => Ok(glib_to_lua(this.lua, this.widget.property_value(prop.name()))),
                None => {
                    println!("Property '{}' not found", &property_name);
                    Err(LuaError::ExternalError(Arc::new(crate::error::Error::PropertyNotFound(property_name))))
                }
            },
        );

        methods.add_method(
            super::SET_PROPERTY,
            |_, this, (property_name, property_value): (String, LuaValue)| {
                let value = match lua_to_glib(&property_value) {
                    Some(value) => value,
                    None => {
                        println!(
                            "Failed to convert property value to glib: {:?}",
                            &property_value
                        );
                        return Err(LuaError::ExternalError(Arc::new(
                            crate::error::Error::NoConversionError,
                        )));
                    }
                };

                if let Err(err) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    this.widget.set_property_from_value(&property_name, &value);
                }))
                {
                    let s = format!("Failed to set property '{}': {:?}", &property_name, err);
                    println!("{}", &s);
                    return Err(LuaError::ExternalError(Arc::new(crate::error::Error::Any(s))));
                }

                Ok(())
            },
        );

        methods.add_method(super::GET_TEXT, |_, this, ()| {
            // TODO: work for more than Entry widgets?
            if let Ok(entry) = this.widget.clone().downcast::<gtk::Entry>() {
                let text = entry.buffer().text();
                Ok(text.to_string())
            } else {
                Err(LuaError::ExternalError(Arc::new(
                    super::Error::UnsupportedOperation,
                )))
            }
        });

        methods.add_method(super::SET_SENSITIVE, |_, this, sensitive: bool| {
            this.widget.set_sensitive(sensitive);
            Ok(())
        });

        methods.add_method(super::SET_LABEL, |_, this, label: String| {
            if let Ok(button) = this.widget.clone().downcast::<gtk::Button>() {
                button.set_label(&label);
                Ok(())
            } else {
                Err(LuaError::ExternalError(Arc::new(
                    super::Error::UnsupportedOperation,
                )))
            }
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

struct Response {
    status_code: u16,
    body: Option<String>,
}

impl Response {
    fn new(r: reqwest::blocking::Response) -> Self {
        Self {
            status_code: r.status().as_u16(),
            body: r.text().ok(),
        }
    }
}

impl LuaUserData for Response {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("status_code", |_, this| Ok(this.status_code));
        fields.add_field_method_get("body", |_, this| Ok(this.body.clone()));
    }
}

#[allow(dead_code)]
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
    use glib::value::ToValue;

    #[test]
    pub fn test_glib_value_to_lua() {
        // TODO: finish adding types, and figure out if it's possible to do null
        let lua = Box::leak(Box::new(Lua::new()));
        assert_eq!(
            glib_to_lua(lua, true.to_value()),
            Some(LuaValue::Boolean(true))
        );
        assert_eq!(
            glib_to_lua(lua, false.to_value()),
            Some(LuaValue::Boolean(false))
        );
    }
}
