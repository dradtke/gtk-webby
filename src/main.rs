use gtk::gdk;
use gtk::prelude::*;

mod error;
mod script;
mod ui;
mod util;
mod window;

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
    app.connect_activate(window::Window::new);

    app.run();
}
