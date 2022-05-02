use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use glib::{clone, Continue, MainContext, PRIORITY_DEFAULT};

pub struct Window {
    #[allow(dead_code)]
    app_window: gtk::ApplicationWindow,
    address_bar: gtk::Entry,
    content: gtk::ScrolledWindow,
    pub state: RefCell<State>,
}

pub struct State {
    pub globals: &'static crate::Globals,
    pub location: String,
    pub http_client: reqwest::blocking::Client,
    pub builder: gtk::Builder,
    user_styles: Option<gtk::CssProvider>,
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
        let http_client = reqwest::blocking::Client::builder().cookie_store(true).build().expect("failed to build http client");

        let builder = gtk::Builder::new();
        let user_styles = None;
        let state = State{globals, location, http_client, builder, user_styles};
        let window = Rc::new(Self{app_window, address_bar, content, state: RefCell::new(state)});

        crate::script::lua::init(window.clone());

        window.clone().address_bar.connect_activate(move |_| {
            let window = window.clone();
            let location = window.address_bar.text().to_string();
            window.go(location);
        });
    }

    fn go(self: Rc<Self>, location: String) {
        println!("Navigating to: {}", &location);
        let request = self.state.borrow().http_client.get(&location);
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        std::thread::spawn(move || {
            let response_result = request.send();
            if let Err(err) = sender.send(response_result) {
                println!("Failed to send response on channel: {}", err);
            }
        });

        receiver.attach(None, clone!(@strong self as window => move |response_result| {
            let window = window.clone();
            let r#do = move || -> crate::Result<()> {
                let response = response_result?;
                window.content.set_child(gtk::Widget::NONE);

                window.state.borrow_mut().location = response.url().to_string();
                let def = crate::ui::Definition::new(response)?;

                // Remove existing user-requested CSS styling, if there is any.
                if let Some(user_styles) = window.state.borrow().user_styles.as_ref() {
                    gtk::StyleContext::remove_provider_for_display(
                        &window.app_window.display(),
                        user_styles);
                }
                window.state.borrow_mut().user_styles = None;

                // If the new page has styles, apply them.
                if !def.styles.is_empty() {
                    let user_styles = gtk::CssProvider::new();
                    user_styles.load_from_data(def.styles.as_bytes());
                    gtk::StyleContext::add_provider_for_display(
                        &window.app_window.display(),
                        &user_styles,
                        gtk::STYLE_PROVIDER_PRIORITY_USER
                    );
                    window.state.borrow_mut().user_styles = Some(user_styles);
                }

                // Set the window title to that requested by the user, or the location if there was
                // none.
                window.app_window.set_title(Some(&def.title.unwrap_or(window.state.borrow().location.clone())));

                // Construct the GTK builder from the UI definition.
                let builder = gtk::Builder::new();
                builder.add_from_string(&def.buildable)?;

                // Find the "body" widget, and set it as the window's content.
                match builder.object::<gtk::Widget>("body") {
                    Some(body) /* once told me */ => window.content.set_child(Some(&body)),
                    None => println!("No object found named 'body'"),
                }

                // Set up callbacks for any href attributes.
                for (object_id, target) in &def.hrefs {
                    let window = window.clone();
                    let target = target.clone();
                    // ???: what other widget types can be clicked?
                    match builder.object::<gtk::Button>(object_id) {
                        Some(widget) => {
                            widget.connect_clicked(move |_| {
                                window.clone().href(&target);
                            });
                        },
                        None => println!("href: no object with id, or object is of the wrong type: {}", object_id),
                    }
                }

                window.state.borrow_mut().builder = builder;

                // Run any defined scripts.
                for script in &def.scripts {
                    script.execute(&window);
                }

                // Clean up any old Lua registry values, such as now-unreferenced callbacks.
                window.state.borrow().globals.lua.expire_registry_values();

                Ok(())
            };

            if let Err(err) = r#do() {
                println!("Navigation error: {}", err);
            }
            Continue(false)
        }));
    }

    fn href(self: Rc<Self>, target: &String) {
        let location = crate::util::absolutize_url(&self.state.borrow().location, target);
        self.address_bar.set_text(&location);
        self.go(location);
    }

    pub fn reload(self: Rc<Self>) {
        let location = self.state.borrow().location.clone();
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
