use crate::window::Window;
use std::rc::Rc;

pub fn register_globals(window: Rc<Window>) {
    let lua = &window.state.borrow().lua;
    let globals = lua.globals();
    {
        // Lotta cloning here, not sure how to clean it up.
        let window = window.clone();
        let alert = lua.create_function(move |_, text: String| {
            window.clone().alert(&text);
            Ok(())
        }).unwrap();
        globals.set("alert", alert).unwrap();
    }
}
