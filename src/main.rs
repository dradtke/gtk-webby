use gtk::gdk;
use gtk::gio;
use gtk::glib;
use glib::clone;
use gtk::prelude::*;
use mlua::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

mod actions;
mod error;
mod headers;
mod script;
mod ui;
mod util;
mod window;

type Result<T> = core::result::Result<T, error::Error>;

pub struct Globals {
    root_certs: Vec<reqwest::tls::Certificate>,
    lua: Lua
}

// TODO: return a result type
fn load_cert(path: &str) -> reqwest::tls::Certificate {
    println!("Loading root cert: {}", path);
    let mut buf = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();
    reqwest::Certificate::from_pem(&buf).unwrap()
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.damienradtke.webby")
        // For some reason, this results in an assertion error that the application isn't
        // registered
        //.menubar(&build_menu())
        .build();

    app.add_main_option(
        "root-certificate",
        glib::Char::from(b'r'),
        glib::OptionFlags::NONE,
        glib::OptionArg::StringArray, // FilenameArray results in strings with null bytes, for some rason
        "Add a root certificate",
        None
    );

    define_actions(&app);

    let root_certs = Rc::new(RefCell::new(vec![]));

    app.connect_handle_local_options(clone!(@strong root_certs => move |_app, dict| {
        match dict.lookup::<Vec<String>>("root-certificate") {
            Ok(Some(paths)) => {
                for path in paths {
                    let cert = load_cert(&path);
                    root_certs.borrow_mut().push(cert);
                }
            },
            Ok(None) => (),
            Err(err) => eprintln!("{}", err),
        }
        -1
    }));

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

    app.connect_activate(move |app| {
        // glib callbacks need referenced values to be 'static.
        let globals = Box::leak(Box::new(Globals{
            root_certs: root_certs.borrow().clone(),
            lua: Lua::new()
        }));
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
