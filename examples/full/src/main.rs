#[macro_use] extern crate rocket;
use rocket_dyn_templates::Template;

#[get("/")]
fn index() -> Template {
    Template::render("index", ())
}

#[launch]
fn rocket() -> _ {
    let mut config = rocket::Config::default();
    config.port = 8008;

    rocket::build().mount("/", routes![index])
        .attach(Template::fairing())
        .configure(rocket::Config{
            port: 8005,
            ..Default::default()
        })
}
