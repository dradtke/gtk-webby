// This module implements the WebDriver spec.
// See: https://www.w3.org/TR/webdriver2/

use gtk::glib;
use webdriver::server::{WebDriverHandler, Session, SessionTeardownKind, Listener};
use webdriver::command::{WebDriverCommand, WebDriverMessage};
use webdriver::error::WebDriverResult;
use webdriver::response::{self, WebDriverResponse};

struct Handler;

impl WebDriverHandler for Handler {
    fn handle_command(&mut self, session: &Option<Session>, msg: WebDriverMessage) -> WebDriverResult<WebDriverResponse> {
        println!("handling command: {:?}", &msg);

        match msg.command {
            WebDriverCommand::NewSession(params) => {
                let session_id = glib::uuid_string_random().as_str().into();
                let capabilities = ();
                Ok(WebDriverResponse::NewSession(response::NewSessionResponse::new(session_id, capabilities.into())))
            },
            WebDriverCommand::DeleteSession => {
                Ok(WebDriverResponse::DeleteSession)
            },
            _ => todo!(),
        }
    }

    fn teardown_session(&mut self, kind: SessionTeardownKind) {
    }
}

pub fn run(addr: &str) -> crate::Result<Listener> {
    webdriver::server::start(
        addr.parse()?,
        vec![url::Host::Domain(String::from("localhost"))], // allowed hosts
        vec![], // allowed origins
        Handler,
        vec![], // extension routes
    ).map_err(crate::error::Error::from)
}
