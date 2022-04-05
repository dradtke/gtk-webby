#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> String {
    // TODO: define Content-Type
    std::fs::read_to_string("src/index.ui").unwrap()
}

#[get("/about")]
fn about() -> String {
    // TODO: define Content-Type
    std::fs::read_to_string("src/about.ui").unwrap()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, about])
}
