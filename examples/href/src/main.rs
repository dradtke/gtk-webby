#[macro_use] extern crate rocket;
use rocket::http::ContentType;

#[get("/")]
fn index() -> (ContentType, String) {
    (ContentType::new("application", "gtk"), std::fs::read_to_string("src/index.ui").unwrap())
}

#[get("/about")]
fn about() -> (ContentType, String) {
    (ContentType::new("application", "gtk"), std::fs::read_to_string("src/about.ui").unwrap())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, about])
        .configure(rocket::Config{
            port: 8001,
            ..Default::default()
        })
}
