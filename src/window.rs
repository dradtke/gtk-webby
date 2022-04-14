use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use mlua::prelude::*;

pub struct Window {
    #[allow(dead_code)]
    app_window: gtk::ApplicationWindow,
    address_bar: gtk::Entry,
    content: gtk::ScrolledWindow,
    pub state: RefCell<State>,
}

pub struct State {
    pub globals: &'static crate::Globals,
    location: String,
    http_client: reqwest::blocking::Client,
    pub builder: gtk::Builder,
}

impl Window {
    pub fn new(app: &gtk::Application, globals: &'static crate::Globals) {
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

        let app_window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Hello World")
            .child(&vbox)
            .width_request(600)
            .height_request(400)
            .build();

        app_window.present();

        let location = String::from("");
        let http_client = reqwest::blocking::Client::new();

        let builder = gtk::Builder::new();
        let state = State{globals, location, http_client, builder};
        let window = Rc::new(Self{app_window, address_bar, content, state: RefCell::new(state)});

        crate::script::lua::init(window.clone());

        window.clone().address_bar.connect_activate(move |_| {
            let window = window.clone();
            let location = window.address_bar.text().to_string();
            window.go(location);
        });
    }

    fn go(self: Rc<Self>, location: String) {
        let r#do = move |location| -> crate::Result<()> {
            self.content.set_child(gtk::Widget::NONE);

            self.state.borrow_mut().location = location;
            println!("Navigating to: {}", &self.state.borrow().location);
            let response = self.state.borrow().http_client.get(&self.state.borrow().location).send()?;
            let def = crate::ui::Definition::new(response)?;

            self.app_window.set_title(Some(&def.title.unwrap_or(self.state.borrow().location.clone())));

            let builder = gtk::Builder::new();
            builder.add_from_string(&def.buildable)?;

            match builder.object::<gtk::Widget>("body") {
                Some(body) /* once told me */ => self.content.set_child(Some(&body)),
                None => println!("No object found named 'body'"),
            }

            for (object_id, target) in &def.hrefs {
                let window = self.clone();
                let target = target.clone();
                // ???: what widget types can be clicked?
                match builder.object::<gtk::Button>(object_id) {
                    Some(widget) => {
                        widget.connect_clicked(move |_| {
                            window.clone().href(&target);
                        });
                    },
                    None => println!("href: no object with id, or object is of the wrong type: {}", object_id),
                }
            }

            self.state.borrow_mut().builder = builder;

            for script in &def.scripts {
                script.execute(&self);
            }

            Ok(())
        };

        if let Err(err) = r#do(location) {
            println!("Navigation error: {}", err);
        }
    }

    fn href(self: Rc<Self>, target: &String) {
        let location = crate::util::absolutize_url(&self.state.borrow().location, target);
        self.address_bar.set_text(&location);
        self.go(location);
    }

    pub fn alert(self: Rc<Self>, text: &str) {
        let dialog = gtk::Dialog::builder()
            .title("Alert")
            .child(&gtk::Label::new(Some(text)))
            .transient_for(&self.app_window)
            .build();
        dialog.present();
    }
}
