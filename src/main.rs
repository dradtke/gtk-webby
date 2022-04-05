use gtk::{gdk, glib};
use glib::clone;
use gtk::prelude::*;

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

#[derive(Clone)]
struct WindowState {
    #[allow(dead_code)]
    window: gtk::ApplicationWindow,
    location: String,
    address_bar: gtk::Entry,
    content: gtk::ScrolledWindow,
    http_client: reqwest::blocking::Client,
}

impl WindowState {
    fn setup_callbacks(self) {
        self.address_bar.connect_activate(clone!(@strong self as this => move |_| {
            let location = this.address_bar.text().to_string();
            WindowState::go(this.clone(), location);
        }));
    }

    fn go(self, location: String) {
        if let Err(err) = self.do_go(location) {
            println!("Failed to go: {}", err);
        }
    }

    fn do_go(mut self, location: String) -> Result<()> {
        self.content.set_child(gtk::Widget::NONE);

        self.location = location;
        println!("Navigating to: {}", &self.location);
        let response = self.http_client.get(&self.location).send()?;
        let def = ui::Definition::new(response)?;

        let builder = gtk::Builder::new();
        builder.add_from_string(&def.buildable)?;

        match builder.object::<gtk::Widget>("body") {
            Some(body) /* once told me */ => self.content.set_child(Some(&body)),
            None => println!("No object found named 'body'"),
        }

        for (object_id, target) in &def.hrefs {
            let target = target.clone();
            // ???: what widget types can be clicked?
            match builder.object::<gtk::Button>(object_id) {
                Some(widget) => {
                    widget.connect_clicked(clone!(@strong self as this => move |_| {
                        WindowState::href(this.clone(), &target);
                    }));
                },
                None => println!("href: no object with id, or object is of the wrong type: {}", object_id),
            }
        }

        Ok(())
    }

    fn href(self, target: &String) {
        let location = WindowState::absolutize_url(&self.location, target);
        self.address_bar.set_text(&location);
        self.go(location);
    }

    fn absolutize_url(current_location: &String, target: &String) -> String {
        if target.contains("://") {
            return target.clone();
        }
        match current_location.find("://") {
            Some(idx) => {
                let mut result = String::new();
                if let Some(root) = current_location[idx+3..].find("/") {
                    result.push_str(&current_location[0..root+idx+3]);
                } else {
                    result.push_str(&current_location);
                }
                result.push_str(target);
                result
            },
            None => unimplemented!(),
        }
    }
}

fn build_ui(app: &gtk::Application) {
    let address_bar = gtk::Entry::new();
    address_bar.set_text("http://localhost:8000"); // for testing

    let content = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

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

    let window_state = WindowState{window, location, address_bar, content, http_client};
    WindowState::setup_callbacks(window_state);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_absolutize_url() {
        assert_eq!(WindowState::absolutize_url(&String::new(), &String::from("http://localhost:8000")), "http://localhost:8000");
        assert_eq!(WindowState::absolutize_url(&String::from("http://localhost:8000"), &String::from("/sub-page")), "http://localhost:8000/sub-page");
        assert_eq!(WindowState::absolutize_url(&String::from("http://localhost:8000/sub-page"), &String::from("/another-page")), "http://localhost:8000/another-page");
    }
}
