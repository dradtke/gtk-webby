use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::{Map, Value};

pub fn main() {
    let client = Client::new("http://localhost:8001");
    let session_id = client.new_session(webdriver::capabilities::SpecNewSessionParameters{
        alwaysMatch: Map::new(),
        firstMatch: vec![],
    });

    client.get(&session_id, "http://localhost:8000/");
    
    client.delete_session(&session_id);
}

pub struct Client {
    base: reqwest::Url,
    http_client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(base: &str) -> Self {
        let http_client = reqwest::blocking::Client::builder()
            .user_agent("WebDriver Client")
            .build()
            .expect("failed to build http client");

        Client {
            base: reqwest::Url::parse(base).expect("invalid base url"),
            http_client
        }
    }

    fn post<Req, Resp>(&self, url: &str, body: Req) -> Resp
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        self.http_client
            .post(self.base.join(url).unwrap())
            .json(&body)
            .send()
            .expect("failed to send POST request")
            .json()
            .expect("failed to parse JSON response")
    }

    fn delete<Resp>(&self, url: &str) -> Resp
    where
        Resp: DeserializeOwned,
    {
        self.http_client
            .delete(self.base.join(url).unwrap())
            .send()
            .expect("failed to send DELETE request")
            .json()
            .expect("failed to parse JSON response")
    }

    pub fn new_session(&self, params: webdriver::capabilities::SpecNewSessionParameters) -> String {
        let response: Map<String, Value> = self.post("/session", params);
        let session_id = response.get("value").unwrap().get("sessionId").unwrap().as_str().unwrap();
        session_id.into()
    }

    pub fn get<S: Into<String>>(&self, session_id: &str, url: S) {
        let params = webdriver::command::GetParameters{url: url.into()};
        let response: Map<String, Value> = self.post(&format!("/session/{}/url", &session_id), params);
        //println!("get response: {:?}", &response);
    }

    pub fn delete_session(&self, session_id: &str) {
        self.delete::<Map<String, Value>>(&format!("/session/{}", session_id));
    }
}
