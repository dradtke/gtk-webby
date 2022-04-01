use gtk::{gdk, glib};
use glib::clone;
use gtk::prelude::*;
use std::rc::Rc;

mod error;
mod util;

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
    address_bar: gtk::Entry,
    content: gtk::ScrolledWindow,
    http_client: reqwest::blocking::Client,
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

    fn do_go(&self) -> Result<()> {
        println!("You entered: {}", self.address_bar.text());
        self.content.set_child(gtk::Widget::NONE);

        let text = self.address_bar.text();
        let response = self.http_client.get(text.as_str()).send()?;
        //let response_body = response.text()?;

        // remove web: attributes before passing to the UI parser

        let builder = gtk::Builder::new();
        //builder.add_from_string(&response_body)?;

        match builder.object::<gtk::Widget>("body") {
            Some(body) /* once told me */ => self.content.set_child(Some(&body)),
            None => println!("No object found named 'body'"),
        }

        for ob in builder.objects() {
        }

        Ok(())
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

    let http_client = reqwest::blocking::Client::new();

    let window_state = Rc::new(WindowState{window, address_bar, content, http_client});
    WindowState::setup_callbacks(window_state);
}
