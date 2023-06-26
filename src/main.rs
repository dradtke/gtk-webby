use gtk::gdk;
use gtk::gio;
use gtk::prelude::*;
use mlua::prelude::*;

mod error;
mod script;
mod ui;
mod util;
mod window;

type Result<T> = core::result::Result<T, error::Error>;

pub struct Globals {
    lua: Lua
}

fn main() {
    // glib callbacks need referenced values to be 'static.
    let globals = Box::leak(Box::new(Globals{
        lua: Lua::new()
    }));

    let app = gtk::Application::builder()
        .application_id("com.damienradtke.webby")
        // For some reason, this results in an assertion error that the application isn't
        // registered
        //.menubar(&build_menu())
        .build();

    app.connect_startup(|app| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_bytes!("style.css"));

        gtk::StyleContext::add_provider_for_display(
            &gdk::Display::default().expect("could not connect to a display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
        );

        app.set_menubar(Some(&build_menu()));
    });
    app.connect_activate(|app| window::Window::new(app, globals));

    app.run();
}

fn build_menu() -> gio::Menu {
    let file = gio::Menu::new();
    let exit = gio::MenuItem::new(Some("Exit"), None);
    file.append_item(&exit);

    let menu = gio::Menu::new();
    menu.append_submenu(Some("File"), &file);
    menu
}
