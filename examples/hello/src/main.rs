#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    return r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <interface>
            <object class="GtkBox" id="body">
                <property name="orientation">vertical</property>
                <property name="halign">start</property>
                <child>
                    <object class="GtkButton">
                        <property name="label">Click Me</property>
                    </object>
                </child>
            </object>
        </interface>
    "#;
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
        .configure(rocket::Config{
            port: 8000,
            ..Default::default()
        })
}
