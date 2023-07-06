#[macro_use] extern crate rocket;
use rocket::http::ContentType;

#[get("/")]
fn index() -> (ContentType, String) {
    (ContentType::new("application", "gtk"), std::fs::read_to_string("src/index.ui").unwrap())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
        .configure(rocket::Config{
            port: 8004,
            ..Default::default()
        })
}
