use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};

const PLACEHOLDER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <requires lib="gtk" version="4.0" />
  <!--
    Widget definitions go here. One widget must have the id "body",
    which indicates the one that should be rendered.
  -->
</interface>
"#;

// Inspired by https://gitlab.gnome.org/World/Rust/sourceview5-rs/-/blob/main/demo/src/main.rs

pub struct Editor {
    window: gtk::Window,
    _view: View,
}

impl Editor {
    pub fn new<F: Fn(String) -> () + 'static>(
        parent: &impl IsA<gtk::Window>,
        starting_text: Option<String>,
        render_callback: F,
    ) -> Self {
        let buffer = Buffer::new(None);
        buffer.set_highlight_syntax(true);
        match starting_text {
            Some(text) => buffer.set_text(&text),
            None => buffer.set_text(&PLACEHOLDER),
        }

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
        view.set_vexpand(true);
        view.set_top_margin(6);

        let render = gtk::Button::with_label("Render");
        render.connect_clicked(clone!(@weak view => move |_| {
            let buffer = view.buffer();
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            render_callback(text.to_string());
        }));

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.append(&view);
        container.append(&render);

        let window = gtk::Window::builder()
            .width_request(400)
            .height_request(300)
            .transient_for(parent)
            .child(&container)
            .build();

        Editor {
            window,
            _view: view,
        }
    }

    pub fn show(&self) {
        self.window.show();
    }
}
