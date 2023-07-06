use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use glib::{clone, Continue, MainContext, PRIORITY_DEFAULT};

pub struct Window {
    #[allow(dead_code)]
    app_window: gtk::ApplicationWindow,
    back_button: gtk::Button,
    forward_button: gtk::Button,
    refresh_button: gtk::Button,
    bookmark_button: gtk::Button,
    address_entry: gtk::Entry,
    content: gtk::ScrolledWindow,
    info_bar: gtk::InfoBar,
    info_bar_text: gtk::Label,
    status_label: gtk::Label,
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
        // Icon names are documented here: https://specifications.freedesktop.org/icon-naming-spec/icon-naming-spec-latest.html
        let back_button = gtk::Button::from_icon_name("go-previous");
        let forward_button = gtk::Button::from_icon_name("go-next");
        let refresh_button = gtk::Button::from_icon_name("view-refresh");
        let bookmark_button = gtk::Button::from_icon_name("bookmark-new"); // no idea where this one is documented

        let address_entry = gtk::Entry::new();
        address_entry.set_property("placeholder-text", "Enter URL");
        address_entry.set_hexpand(true);
        //address_entry.set_text("http://localhost:8000"); // for testing

        let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        top_bar.append(&back_button);
        top_bar.append(&forward_button);
        top_bar.append(&refresh_button);
        top_bar.append(&address_entry);
        top_bar.append(&bookmark_button);

        let content = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let info_bar = gtk::InfoBar::builder().revealed(false).show_close_button(true).build();
        info_bar.connect_response(move |this, response| {
            match response {
                gtk::ResponseType::Close => this.set_revealed(false),
                _ => (),
            }
        });
        let info_bar_text = gtk::Label::new(None);
        info_bar.add_child(&info_bar_text);

        let status_label = gtk::Label::new(None);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
        vbox.append(&top_bar);
        vbox.append(&content);
        vbox.append(&info_bar);
        vbox.append(&status_label);

        let app_window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Webby")
            .child(&vbox)
            .width_request(800)
            .height_request(600)
            .show_menubar(true)
            .build();

        app_window.present();

        let location = String::from("");
        let mut http_client_builder = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .user_agent("GTK Webby")
            .default_headers(crate::headers::request_headers());

        for cert in &globals.root_certs {
            http_client_builder = http_client_builder.add_root_certificate(cert.clone());
        }

        let http_client = http_client_builder
            .build()
            .expect("failed to build http client");

        let builder = gtk::Builder::new();
        let user_styles = None;
        let state = State{globals, location, http_client, builder, user_styles};
        let window = Rc::new(Self{
            app_window,
            back_button,
            forward_button,
            refresh_button,
            bookmark_button,
            address_entry,
            content,
            info_bar,
            info_bar_text,
            status_label,
            state: RefCell::new(state),
        });

        crate::script::lua::init(window.clone());

        window.back_button.connect_clicked(clone!(@weak window => move |_| {
            eprintln!("TODO: go back");
        }));

        window.forward_button.connect_clicked(clone!(@weak window => move |_| {
            eprintln!("TODO: go forward");
        }));

        window.refresh_button.connect_clicked(clone!(@weak window => move |_| {
            window.refresh();
        }));

        window.bookmark_button.connect_clicked(clone!(@weak window => move |_| {
            eprintln!("TODO: add bookmark");
        }));

        window.address_entry.connect_activate(clone!(@weak window => move |_| {
            let location = window.address_entry.text().to_string();
            window.go(location);
        }));
    }

    fn refresh(self: Rc<Self>) {
        let location = self.state.borrow().location.clone();
        self.go(location);
    }

    fn go(self: Rc<Self>, location: String) {
        self.info_bar.set_revealed(false);

        //println!("Navigating to: {}", &location);
        self.status_label.set_label(&format!("Loading {}...", &location));
        let request = self.state.borrow().http_client.get(&location);
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        std::thread::spawn(move || {
            let response_result = request.send();
            if let Err(err) = sender.send(response_result) {
                println!("Failed to send response on channel: {}", err);
            }
        });

        receiver.attach(None, clone!(@weak self as window => @default-return Continue(false), move |response_result| {
            let r#do = || -> crate::Result<()> {
                let response = response_result?;
                window.content.set_child(gtk::Widget::NONE);
                window.state.borrow_mut().location = response.url().to_string();

                let mime_type: mime::Mime = match response.headers().get(reqwest::header::CONTENT_TYPE) {
                    Some(content_type) => content_type.to_str()?.parse()?,
                    None => return Err(crate::error::Error::NoContentTypeError),
                };

                match mime_type.type_() {
                    mime::TEXT => window.clone().render_text(response),
                    mime::APPLICATION if mime_type.subtype() == "gtk" => window.clone().render_gtk(response),
                    _ => Err(crate::error::Error::UnsupportedContentTypeError(mime_type.essence_str().to_string())),
                }
            };

            if let Err(err) = r#do() {
                let err_text = err.to_string().replace(": ", ":\n");
                window.info_bar_text.set_text(&err_text);
                window.info_bar.set_message_type(gtk::MessageType::Error);
                window.info_bar.set_revealed(true);
                println!("Navigation error: {}", err);
            }

            window.status_label.set_text("");
            Continue(false)
        }));
    }

    fn render_text<R: Read>(self: Rc<Self>, mut r: R) -> crate::Result<()> {
        let mut s = String::new();
        r.read_to_string(&mut s)?;
        self.content.set_child(Some(&gtk::TextView::with_buffer(&gtk::TextBuffer::builder().text(&s).build())));
        Ok(())
    }

    fn render_gtk<R: Read>(self: Rc<Self>, r: R) -> crate::Result<()> {
        let def = crate::ui::Definition::new(r)?;

        // Remove existing user-requested CSS styling, if there is any.
        if let Some(user_styles) = self.state.borrow().user_styles.as_ref() {
            gtk::StyleContext::remove_provider_for_display(
                &self.app_window.display(),
                user_styles);
        }
        self.state.borrow_mut().user_styles = None;

        // If the new page has styles, apply them.
        if !def.styles.is_empty() {
            let user_styles = gtk::CssProvider::new();
            user_styles.load_from_data(def.styles.as_bytes());
            gtk::StyleContext::add_provider_for_display(
                &self.app_window.display(),
                &user_styles,
                gtk::STYLE_PROVIDER_PRIORITY_USER
            );
            self.state.borrow_mut().user_styles = Some(user_styles);
        }

        // Set the window title to that requested by the user, or the location if there was
        // none.
        self.app_window.set_title(Some(&def.title.unwrap_or(self.state.borrow().location.clone())));

        // Construct the GTK builder from the UI definition.
        let builder = gtk::Builder::new();
        builder.add_from_string(&def.buildable)?;

        // Find the "body" widget, and set it as the window's content.
        match builder.object::<gtk::Widget>("body") {
            Some(body) /* once told me */ => self.content.set_child(Some(&body)),
            None => println!("No object found named 'body'"),
        }

        // Set up callbacks for any href attributes.
        for (object_id, target) in &def.hrefs {
            let window = self.clone();
            let target = target.clone();
            match builder.object::<gtk::Widget>(object_id) {
                Some(widget) => {
                    widget.connect_local("clicked", false, move |_| {
                        window.clone().href(&target);
                        None
                    });
                },
                None => println!("href: no object with id, or object is of the wrong type: {}", object_id),
            }
        }

        self.state.borrow_mut().builder = builder;

        // Run any defined scripts.
        for script in &def.scripts {
            script.execute(&self);
        }

        // Clean up any old Lua registry values, such as now-unreferenced callbacks.
        self.state.borrow().globals.lua.expire_registry_values();

        Ok(())
    }

    fn href(self: Rc<Self>, target: &String) {
        let location = crate::util::absolutize_url(&self.state.borrow().location, target);
        self.address_entry.set_text(&location);
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
