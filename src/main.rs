use gtk::{gdk, glib};
use glib::clone;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

mod error;
mod ui;

type Result<T> = core::result::Result<T, error::Error>;

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.damienradtke.webish-client")
        .build();

    app.connect_startup(|_| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_bytes!("style.css"));

        gtk::StyleContext::add_provider_for_display(
            &gdk::Display::default().expect("could not connect to a display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
    });
    app.connect_activate(build_ui);

    app.run();
}

struct WindowState {
    window: gtk::ApplicationWindow,
    location: String,
    address_bar: gtk::Entry,
    content: gtk::ScrolledWindow,
    http_client: reqwest::blocking::Client,
}

impl WindowState {
    fn setup_callbacks(window_state: Rc<RefCell<WindowState>>) {
        window_state.borrow().address_bar.connect_activate(clone!(@strong window_state => move |_| {
            WindowState::go(window_state.clone());
        }));
    }

    fn go(window_state: Rc<RefCell<WindowState>>) {
        if let Err(err) = WindowState::do_go(window_state) {
            println!("Failed to go: {}", err);
        }
    }

    fn do_go(window_state: Rc<RefCell<WindowState>>) -> Result<()> {
        let mut this = window_state.borrow_mut();
        this.content.set_child(gtk::Widget::NONE);

        this.location = this.address_bar.text().to_string();
        println!("Navigating to: {}", this.location);
        let response = this.http_client.get(&this.location).send()?;
        let def = ui::Definition::new(response)?;

        let builder = gtk::Builder::new();
        builder.add_from_string(&def.buildable)?;

        match builder.object::<gtk::Widget>("body") {
            Some(body) /* once told me */ => this.content.set_child(Some(&body)),
            None => println!("No object found named 'body'"),
        }

        for (object_id, target) in &def.hrefs {
            let target = target.clone();
            // TODO: what widget types can be clicked?
            match builder.object::<gtk::Button>(object_id) {
                Some(widget) => {
                    // There has to be a better way to do this...
                    let window_state = window_state.clone();
                    widget.connect_clicked(move |_| {
                        let window_state = window_state.clone();
                        WindowState::href(window_state, &target);
                    });
                },
                None => println!("href: no object with id, or object is of the wrong type: {}", object_id),
            }
        }

        Ok(())
    }

    fn href(window_state: Rc<RefCell<WindowState>>, target: &String) {
        println!("Hrefing to {}", target);
    }
}

fn build_ui(app: &gtk::Application) {
    let address_bar = gtk::Entry::new();
    address_bar.set_text("http://localhost:8000"); // for testing

    let content = gtk::ScrolledWindow::new();
    content.set_child(Some(
        &gtk::Label::builder()
        .css_classes(vec!["placeholder".to_string()])
        .label("Enter an address")
        .build())
    );

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
    vbox.append(&address_bar);
    vbox.append(&content);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Hello World")
        .child(&vbox)
        .width_request(400)
        .height_request(300)
        .build();

    window.present();

    let location = String::from("");
    let http_client = reqwest::blocking::Client::new();

    let window_state = Rc::new(RefCell::new(WindowState{window, location, address_bar, content, http_client}));
    WindowState::setup_callbacks(window_state);
}
