use gtk::{gdk, glib};
use glib::clone;
use gtk::prelude::*;
use std::rc::Rc;
use std::fmt;

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
    address_bar: gtk::Entry,
    content: gtk::ScrolledWindow,
    http_client: reqwest::blocking::Client,
}

enum Error {
    IoError(std::io::Error),
    HttpError(reqwest::Error),
    GlibError(glib::error::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "io error: {}", err),
            Error::HttpError(err) => write!(f, "http error: {}", err),
            Error::GlibError(err) => write!(f, "glib error: {}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::HttpError(err)
    }
}

impl From<glib::Error> for Error {
    fn from(err: glib::Error) -> Error {
        Error::GlibError(err)
    }
}

impl WindowState {
    fn setup_callbacks(window_state: Rc<WindowState>) {
        window_state.address_bar.connect_activate(clone!(@strong window_state => move |_| {
            window_state.go();
        }));
    }

    fn go(&self) {
        if let Err(err) = self.do_go() {
            println!("Failed to go: {}", err);
        }
    }

    fn do_go(&self) -> Result<(), Error> {
        println!("You entered: {}", self.address_bar.text());
        self.content.set_child(gtk::Widget::NONE);

        let text = self.address_bar.text();
        let response = self.http_client.get(text.as_str()).send()?;
        let response_body = response.text()?;

        let builder = gtk::Builder::new();
        builder.add_from_string(&response_body)?;

        match builder.object::<gtk::Widget>("body") {
            Some(body) /* once told me */ => self.content.set_child(Some(&body)),
            None => println!("No object found named 'body'"),
        }

        Ok(())
    }
}

fn build_ui(app: &gtk::Application) {
    let address_bar = gtk::Entry::new();
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

    let http_client = reqwest::blocking::Client::new();

    let window_state = Rc::new(WindowState{window, address_bar, content, http_client});
    WindowState::setup_callbacks(window_state);
}
