use glib::clone;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use mlua::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

mod actions;
mod editor;
mod error;
mod headers;
mod history;
mod script;
mod ui;
mod util;
mod window;

type Result<T> = core::result::Result<T, error::Error>;

pub struct Globals {
    root_certs: Vec<reqwest::tls::Certificate>,
    lua: Lua,
}

fn load_cert(path: &str) -> anyhow::Result<reqwest::tls::Certificate> {
    let mut buf = Vec::new();
    File::open(path)?.read_to_end(&mut buf)?;
    Ok(reqwest::Certificate::from_pem(&buf)?)
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.damienradtke.webby")
        // For some reason, this results in an assertion error that the application isn't
        // registered
        //.menubar(&build_menu())
        .build();

    app.add_main_option(
        "add-root-cert",
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::StringArray, // FilenameArray results in strings with null bytes, for some reason
        "Add a root certificate. Can be specified multiple times",
        Some("path/to/cert.pem"),
    );

    define_app_actions(&app);

    let root_certs = Rc::new(RefCell::new(vec![]));

    app.connect_handle_local_options(clone!(@strong root_certs => move |_app, dict| {
        match dict.lookup::<Vec<String>>("add-root-cert") {
            Ok(Some(paths)) => {
                for path in paths {
                    match load_cert(&path) {
                        Ok(cert) => {
                            println!("Loaded root cert {}", &path);
                            root_certs.borrow_mut().push(cert);
                        },
                        Err(err) => {
                            // TODO: use glib's logging facilities?
                            println!("Failed to load root cert {}: {}", &path, &err);
                        },
                    }
                }
            },
            Ok(None) => (),
            Err(err) => eprintln!("{}", err),
        }
        -1
    }));

    app.connect_startup(|app| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("style.css"));

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("could not connect to a display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        app.set_menubar(Some(&build_menu()));
    });

    app.connect_activate(move |app| {
        // glib callbacks need referenced values to be 'static.
        let globals = Box::leak(Box::new(Globals {
            root_certs: root_certs.borrow().clone(),
            lua: Lua::new(),
        }));
        window::Window::new(app, globals);
    });

    app.run();
}

fn define_app_actions(app: &gtk::Application) {
    let quit = gio::SimpleAction::new("quit", None);
    {
        let app = app.clone();
        quit.connect_activate(move |_, _| app.quit());
    }
    app.add_action(&quit);

    let about = gio::SimpleAction::new("about", None);
    about.connect_activate(actions::about);
    app.add_action(&about);
}

fn build_menu() -> gio::Menu {
    let file = gio::Menu::new();
    let open_source_editor =
        gio::MenuItem::new(Some("Open Source Editor"), Some("win.open-source-editor"));
    let quit = gio::MenuItem::new(Some("Quit"), Some("app.quit"));
    file.append_item(&open_source_editor);
    file.append_item(&quit);

    let help = gio::Menu::new();
    let about = gio::MenuItem::new(Some("About"), Some("app.about"));
    help.append_item(&about);

    let menu = gio::Menu::new();
    menu.append_submenu(Some("File"), &file);
    menu.append_submenu(Some("Help"), &help);
    menu
}
