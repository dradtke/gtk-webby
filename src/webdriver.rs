// This module implements the WebDriver spec.
// See: https://www.w3.org/TR/webdriver2/

use gtk::glib;
use webdriver::server::{WebDriverHandler, Session, SessionTeardownKind, Listener};
use webdriver::command::{WebDriverCommand, WebDriverMessage};
use webdriver::error::WebDriverResult;
use webdriver::response::{self, WebDriverResponse};

struct Handler {
    windows: crate::window::WindowList,
    session_id: Option<String>,
    //get_active_window: Arc<dyn Fn() -> Arc<crate::window::Window> + Send + Sync>,
}

impl Handler {
    /*
    fn new<F: Fn() -> Arc<crate::window::Window> + Send + Sync>(get_active_window: F) -> Self {
        Handler{
            session_id: None,
            get_active_window: Arc::new(get_active_window),
        }
    }
    */
    fn new(windows: crate::window::WindowList) -> Self {
        Handler{
            windows,
            session_id: None,
        }
    }
}

impl WebDriverHandler for Handler {
    fn handle_command(&mut self, _session: &Option<Session>, msg: WebDriverMessage) -> WebDriverResult<WebDriverResponse> {
        println!("handling command: {:?}", &msg);

        match msg.command {
            WebDriverCommand::NewSession(_params) => {
                let session_id: String = glib::uuid_string_random().as_str().into();
                let capabilities = ();
                self.session_id = Some(session_id.clone());
                Ok(WebDriverResponse::NewSession(response::NewSessionResponse::new(session_id, capabilities.into())))
            },
            WebDriverCommand::DeleteSession => {
                Ok(WebDriverResponse::DeleteSession)
            },
            WebDriverCommand::Get(params) => {
                println!("Navigating to {}", &params.url);
                let window = crate::window::Window::active(&self.windows);
                // All operations on the window must happen on the main GTK thread.
                glib::idle_add_once(|| {
                    window.go(params.url, true);
                });
                Ok(WebDriverResponse::Void)
            },
            _ => todo!(),
        }
    }

    fn teardown_session(&mut self, _kind: SessionTeardownKind) {
        self.session_id = None;
    }
}

pub fn run(windows: crate::window::WindowList, addr: &str) -> crate::Result<Listener> {
    webdriver::server::start(
        addr.parse()?,
        vec![url::Host::Domain(String::from("localhost"))], // allowed hosts
        vec![], // allowed origins
        Handler::new(windows),
        vec![], // extension routes
    ).map_err(crate::error::Error::from)
}
