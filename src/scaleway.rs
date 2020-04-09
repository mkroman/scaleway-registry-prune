pub struct Registry {}

pub struct Client {
    client: reqwest::Client,
    auth_token: String,
}

impl Client {
    pub fn new(auth_token: String) -> Self {
        let client = reqwest::Client::new();

        Client { client, auth_token }
    }
}
