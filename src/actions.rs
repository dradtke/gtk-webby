use gtk::gio;
use gtk::glib;

pub fn about(_action: &gio::SimpleAction, _param: Option<&glib::Variant>) {
    eprintln!("Showing the About dialog!");
    let _ = gtk::AboutDialog::builder()
        .authors(vec![String::from("Damien Radtke")])
        .build();
}
