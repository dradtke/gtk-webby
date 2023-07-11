use gtk::prelude::*;
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};

// Inspired by https://gitlab.gnome.org/World/Rust/sourceview5-rs/-/blob/main/demo/src/main.rs

pub struct Editor {
    window: gtk::Window,
    view: View,
}

impl Editor {
    pub fn new(parent: &impl IsA<gtk::Window>) -> Self {
        let buffer = Buffer::new(None);
        buffer.set_highlight_syntax(true);

        if let Some(ref language) = LanguageManager::new().language("xml") {
            buffer.set_language(Some(language));
        }

        if let Some(ref scheme) = StyleSchemeManager::new().scheme("solarized-dark") {
            buffer.set_style_scheme(Some(scheme));
        }

        let view = View::with_buffer(&buffer);
        view.set_monospace(true);
        view.set_show_line_numbers(true);
        view.set_highlight_current_line(true);
        view.set_tab_width(4);
        view.set_hexpand(true);
        view.set_vexpand(true);

        let render = gtk::Button::with_label("Render");
        render.connect_clicked(|_| println!("Rendering from source editor"));

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.append(&view);
        container.append(&render);

        let window = gtk::Window::builder()
            .width_request(400)
            .height_request(300)
            .transient_for(parent)
            .child(&container)
            .build();

        Editor { window, view }
    }

    pub fn show(&self) {
        self.window.show();
    }
}
