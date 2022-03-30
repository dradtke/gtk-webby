#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    // TODO: define Content-Type
    include_str!("index.ui")
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
