#[macro_use] extern crate rocket;
use rocket::form::Form;

#[get("/")]
fn index() -> String {
    // TODO: define Content-Type
    std::fs::read_to_string("src/index.ui").unwrap()
}

#[derive(FromForm)]
struct LoginRequest<'r> {
    username: &'r str,
    password: &'r str,
}

#[post("/", data="<request>")]
fn login(request: Form<LoginRequest<'_>>) -> String {
    println!("logging in with username '{}' and password '{}'", &request.username, &request.password);
    "".to_string()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, login])
}
