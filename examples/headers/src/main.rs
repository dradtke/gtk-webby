#[macro_use]
extern crate rocket;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::request::{self, FromRequest, Request};

struct ClientInfo {
    user_agent: String,
    gtk_version_major: String,
    gtk_version_minor: String,
    gtk_version_micro: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientInfo {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let get_header = |name| {
            request
                .headers()
                .get_one(name)
                .unwrap_or("<not present>")
                .into()
        };

        request::Outcome::Success(ClientInfo {
            user_agent: get_header("user-agent"),
            gtk_version_major: get_header("x-gtk-version-major"),
            gtk_version_minor: get_header("x-gtk-version-minor"),
            gtk_version_micro: get_header("x-gtk-version-micro"),
        })
    }
}

#[get("/")]
fn index(client_info: ClientInfo) -> (ContentType, String) {
    (
        ContentType::new("application", "gtk"),
        format!(
            r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <interface>
            <object class="GtkBox" id="body">
                <property name="orientation">vertical</property>
                <property name="halign">start</property>
                <child>
                    <object class="GtkLabel">
                        <property name="label">User Agent: {user_agent}</property>
                    </object>
                </child>
                <child>
                    <object class="GtkLabel">
                        <property name="label">GTK Version: {gtk_version_major}.{gtk_version_minor}.{gtk_version_micro}</property>
                    </object>
                </child>
            </object>
        </interface>
    "#,
            user_agent = client_info.user_agent,
            gtk_version_major = client_info.gtk_version_major,
            gtk_version_minor = client_info.gtk_version_minor,
            gtk_version_micro = client_info.gtk_version_micro,
        ),
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(AdHoc::on_request("Header Logger", |req, _| {
            Box::pin(async move {
                println!("   Received headers:");
                for header in req.headers().iter() {
                    println!("     {} = {}", header.name, header.value);
                }
            })
        }))
        .configure(rocket::Config {
            port: 8000,
            ..Default::default()
        })
}
