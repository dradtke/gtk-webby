use gtk::gdk;
use gtk::gio;
use gtk::prelude::*;
use mlua::prelude::*;

mod actions;
mod error;
mod headers;
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

    define_actions(&app);

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
    app.connect_activate(|app| {
        window::Window::new(app, globals);
    });

    app.run();
}

fn define_actions(app: &gtk::Application) {
    let quit = gio::SimpleAction::new("quit", None);
    {
        let app = app.clone();
        quit.connect_activate(move |_,_| app.quit());
    }
    app.add_action(&quit);

    let about = gio::SimpleAction::new("about", None);
    about.connect_activate(actions::about);
    app.add_action(&about);
}

fn build_menu() -> gio::Menu {
    let file = gio::Menu::new();
    let quit = gio::MenuItem::new(Some("Quit"), Some("app.quit"));
    file.append_item(&quit);

    let help = gio::Menu::new();
    let about = gio::MenuItem::new(Some("About"), Some("app.about"));
    help.append_item(&about);

    let menu = gio::Menu::new();
    menu.append_submenu(Some("File"), &file);
    menu.append_submenu(Some("Help"), &help);
    menu
}
