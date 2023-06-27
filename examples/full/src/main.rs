#[macro_use] extern crate rocket;
use rocket_dyn_templates::Template;
use rocket::serde::Serialize;

#[derive(Serialize)]
struct PageData {
    posts: Vec<String>,
}

#[get("/")]
fn index() -> Template {
    Template::render("index", PageData{
        posts: vec![
            String::from("This is my first post."),
            String::from("This is my second post."),
        ],
    })
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
