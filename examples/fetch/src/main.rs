#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> String {
    // TODO: define Content-Type
    std::fs::read_to_string("src/index.ui").unwrap()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
        .configure(rocket::Config{
            port: 8004,
            ..Default::default()
        })
}
