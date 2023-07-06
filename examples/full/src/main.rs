#[macro_use] extern crate rocket;
use rocket_dyn_templates::Template;
use rocket::config::TlsConfig;
use rocket::http::ContentType;
use rocket::serde::Serialize;

#[derive(Serialize)]
struct PageData {
    posts: Vec<String>,
}

#[get("/")]
fn index() -> (ContentType, Template) {
    (ContentType::new("application", "gtk"), Template::render("index", PageData{
        posts: vec![
            String::from("This is my first post."),
            String::from("This is my second post."),
        ],
    }))
}

#[launch]
fn rocket() -> _ {
    let tls_config = TlsConfig::from_paths("localhost.crt", "localhost.key");

    rocket::build().mount("/", routes![index])
        .attach(Template::fairing())
        .configure(rocket::Config{
            port: 8005,
            tls: Some(tls_config),
            ..Default::default()
        })
}
