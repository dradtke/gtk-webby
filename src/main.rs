use glib::{clone, ExitCode};
use gtk::gio::Cancellable;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use mlua::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

mod actions;
mod editor;
mod error;
mod headers;
mod history;
mod script;
mod ui;
mod util;
mod webdriver;
mod window;

type Result<T> = core::result::Result<T, error::Error>;

pub struct Globals {
    root_certs: Vec<reqwest::tls::Certificate>,
    lua: Lua,
}

fn load_cert(path: &str) -> Result<reqwest::tls::Certificate> {
    let mut buf = Vec::new();
    File::open(path)?.read_to_end(&mut buf)?;
    Ok(reqwest::Certificate::from_pem(&buf)?)
}

fn main() -> Result<ExitCode> {
    env_logger::init();

    let app = gtk::Application::builder()
        .application_id("com.damienradtke.webby")
        .build();

    app.add_main_option(
        "add-root-cert",
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::StringArray, // FilenameArray results in strings with null bytes, for some reason
        "Add a root certificate. Can be specified multiple times",
        Some("path/to/cert.pem"),
    );

    app.add_main_option(
        "watch",
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::StringArray,
        "Watch a file or directory for changes, and reload the current page when detected",
        Some("path/to/file.ui"),
    );

    app.add_main_option(
        "bind-webdriver",
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::String,
        "Listen for WebDriver requests on the provided address",
        Some("0.0.0.0:8000"),
    );

    let windows: window::WindowList = Arc::new(Mutex::new(vec![]));
    let root_certs = Rc::new(RefCell::new(vec![]));
    let file_monitors = Rc::new(RefCell::new(vec![]));
    let webdriver_listeners = Rc::new(RefCell::new(vec![]));

    app.connect_handle_local_options(
        clone!(@strong windows, @strong root_certs => move |_app, dict| {
            println!("app handle local options");

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

            match dict.lookup::<Vec<String>>("watch") {
                Ok(Some(paths)) => {
                    for path in paths {
                        if let Some(monitor) = watch_path(windows.clone(), &path) {
                            file_monitors.borrow_mut().push(monitor);
                        }
                    }
                },
                Ok(None) => (),
                Err(err) => eprintln!("{}", err),
            }

            match dict.lookup::<String>("bind-webdriver") {
                Ok(Some(addr)) => {
                    match crate::webdriver::run(windows.clone(), &addr) {
                        Ok(listener) => {
                            println!("Listening for WebDriver requests on {}", &addr);
                            webdriver_listeners.borrow_mut().push(listener);
                        },
                        Err(err) => println!("Error starting WebDriver listener: {}", err),
                    }
                },
                Ok(None) => (),
                Err(err) => eprintln!("{}", err),
            }

            -1
        }),
    );

    app.connect_startup(|app| {
        println!("app startup");

        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("style.css"));

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("could not connect to a display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        if let Err(err) = app.register(Cancellable::NONE) {
            println!("Failed to register appplication: {}", err);
        }
        app.set_menubar(Some(&build_menu()));
        define_app_actions(&app);
    });

    app.connect_activate(move |app| {
        println!("app activate");
        // glib callbacks need referenced values to be 'static.
        let globals = Box::leak(Box::new(Globals {
            root_certs: root_certs.borrow().clone(),
            lua: Lua::new(),
        }));
        let window = window::Window::new(app, globals);
        windows.lock().unwrap().push(window);
    });

    Ok(app.run())
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

fn watch_path(
    windows: Arc<Mutex<Vec<Arc<window::Window>>>>,
    path: &str,
) -> Option<gio::FileMonitor> {
    let file = gio::File::for_path(path);
    let monitor = match file.monitor_file(gio::FileMonitorFlags::WATCH_MOVES, Cancellable::NONE) {
        Ok(monitor) => monitor,
        Err(err) => {
            println!("Error creating monitor for path '{}': {}", path, err);
            return None;
        }
    };
    println!("Watching path for changes: {}", path);
    monitor.connect_changed(move |_monitor, file, _other_file, event| match event {
        gio::FileMonitorEvent::AttributeChanged | gio::FileMonitorEvent::Changed => {
            if let Some(path) = file.path() {
                println!("{:?} changed", &path);
            }
            let windows = windows.lock().unwrap();
            for window in windows.iter() {
                window.clone().reload();
            }
        }
        _ => (),
    });
    Some(monitor)
}
