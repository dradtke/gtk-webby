#[macro_use] extern crate rocket;
use rocket::form::Form;
use rocket::http::{CookieJar, Cookie};
use rocket::response::{status, Redirect};
use rocket::serde::Serialize;
use rocket_dyn_templates::Template;

#[derive(Serialize)]
struct UserInfo<'a> {
    username: &'a str,
}

#[get("/")]
fn index(cookies: &CookieJar<'_>) -> Template {
    match cookies.get("username") {
        Some(username) => Template::render("home", UserInfo{username: username.value()}),
        None => Template::render("index", ()),
    }
}

#[derive(FromForm)]
struct LoginRequest<'r> {
    username: &'r str,
    password: &'r str,
}

#[post("/", data="<request>")]
fn login(cookies: &CookieJar<'_>, request: Form<LoginRequest<'_>>) -> status::Accepted<()> {
    cookies.add(Cookie::new("username", request.username.to_string()));
    status::Accepted(None)
}

#[post("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::named("username"));
    Redirect::to(uri!("/"))
}

#[launch]
fn rocket() -> _ {
    let mut config = rocket::Config::default();
    config.port = 8008;

    rocket::build().mount("/", routes![index, login, logout])
        .attach(Template::fairing())
        .configure(rocket::Config{
            port: 8004,
            ..Default::default()
        })
}
