use serde::Deserialize;

use crate::Error;
use std::time::Duration;

pub struct Registry<'a> {
    client: &'a Client,
    namespace: String,
}

pub struct Client {
    client: reqwest::Client,
    region: String,
    auth_token: String,
}

#[derive(Deserialize, Debug)]
pub struct Namespace {
    id: String,
    name: String,
    size: Option<usize>,
    description: String,
    organization_id: String,
    status: String,
    status_message: String,
    endpoint: String,
    is_public: bool,
    // FIXME: Use proper datetime
    created_at: String,
    // FIXME: Use proper datetime
    updated_at: String,
    image_count: usize,
}

#[derive(Deserialize, Debug)]
pub struct NamespaceList {
    namespaces: Vec<Namespace>,
    total_count: usize,
}

#[derive(Deserialize, Debug)]
pub struct ImageList {
    images: Vec<Image>,
    total_count: usize,
}

#[derive(Deserialize, Debug)]
pub struct Image {
    id: String,
    name: String,
    namespace_id: String,
    status: String,
    status_message: Option<String>,
    visibility: String,
    size: usize,
    // FIXME: Use proper datetime
    created_at: String,
    // FIXME: Use proper datetime
    updated_at: String,
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct ErrorMessage {
    message: String,
}

impl<'a> Registry<'a> {
    pub async fn namespaces(&self) -> Result<NamespaceList, Error> {
        let res = self.client.get("/namespaces").send().await?;

        if res.status().is_success() {
            res.json().await.map_err(Into::into)
        } else {
            let err = res.json::<ErrorMessage>().await?;
            Err(Error::ApiError(err.message))
        }
    }

    /// Returns the namespace details for a given `namespace_id`
    pub async fn namespace(&self, namespace_id: &str) -> Result<Namespace, Error> {
        let res = self
            .client
            .get(&format!("/namespaces/{}", namespace_id))
            .send()
            .await?;

        if res.status().is_success() {
            res.json().await.map_err(Into::into)
        } else {
            let err = res.json::<ErrorMessage>().await?;
            Err(Error::ApiError(err.message))
        }
    }

    /// Returns a list of all images accessible to the user
    pub async fn images(&self) -> Result<ImageList, Error> {
        let res = self.client.get("/images").send().await?;

        if res.status().is_success() {
            res.json().await.map_err(Into::into)
        } else {
            let err = res.json::<ErrorMessage>().await?;
            Err(Error::ApiError(err.message))
        }
    }
}

impl<'a> Client {
    /// Returns a new client with a given `auth_token` that operates in `region`
    pub fn new(auth_token: String, region: String) -> Self {
        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Client {
            client,
            auth_token,
            region,
        }
    }

    pub fn registry(&self, namespace: String) -> Registry {
        Registry {
            client: self,
            namespace,
        }
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(&format!(
                "https://api.scaleway.com/registry/v1/regions/{}{}",
                self.region, path
            ))
            .header("X-Auth-Token", &self.auth_token)
    }
}
