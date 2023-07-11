use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

pub fn about(_action: &gio::SimpleAction, _param: Option<&glib::Variant>) {
    gtk::AboutDialog::builder()
        .program_name("Webby")
        .authors(vec![String::from("Damien Radtke <me@damienradtke.com>")])
        .comments("Program to render GTK applications using a web-based deployment model.")
        .website("https://damienradtke.com/post/building-gtk-applications-like-websites/")
        .build()
        .set_visible(true);
}

pub fn open_source_editor(_action: &gio::SimpleAction, _param: Option<&glib::Variant>) {}
