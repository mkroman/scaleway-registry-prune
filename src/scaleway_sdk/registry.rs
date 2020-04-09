use serde::Deserialize;

use crate::scaleway_sdk::Error;
use std::time::Duration;

const DEFAULT_API_ENDPOINT: &'static str = "https://api.scaleway.com/registry/v1";

pub struct Registry {
    client: reqwest::Client,
    region: String,
    endpoint: String,
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

#[allow(dead_code)]
impl Namespace {
    /// Returns the unique id of the namespace
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the name of the namespace
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the total size for all images in this namespace
    ///
    /// Note that the size is only present if the namespace is retrieved using
    /// [`Registry::namespace`]
    ///
    /// [`Registry::namespace`]: struct.Registry.html#method.namespace
    #[allow(dead_code)]
    pub fn size(&self) -> Option<usize> {
        self.size
    }

    /// Returns the user-defined description of the namespace
    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the unique organization id
    pub fn organization_id(&self) -> &str {
        &self.organization_id
    }

    /// Returns the namespace status
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Returns status_message
    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    /// Returns endpoint
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn is_public(&self) -> bool {
        self.is_public
    }

    /// Returns created_at
    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    /// Returns updated_at
    pub fn updated_at(&self) -> &str {
        &self.updated_at
    }

    /// Returns image_count
    pub fn image_count(&self) -> usize {
        self.image_count
    }
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

#[allow(dead_code)]
impl Image {
    /// Returns id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns namespace_id
    pub fn namespace_id(&self) -> &str {
        &self.namespace_id
    }

    /// Returns status
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Returns status_message
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_ref().map(|s| &s[..])
    }

    /// Returns visibility
    pub fn visibility(&self) -> &str {
        &self.visibility
    }

    /// Returns size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns created_at
    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    /// Returns updated_at
    pub fn updated_at(&self) -> &str {
        &self.updated_at
    }

    /// Returns tags
    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}

#[derive(Deserialize, Debug)]
struct ErrorMessage {
    message: String,
}

impl Registry {
    pub fn new(auth_token: String, region: String) -> Self {
        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Registry {
            client,
            endpoint: format!("{}/regions/{}", DEFAULT_API_ENDPOINT, region),
            auth_token,
            region,
        }
    }

    pub fn endpoint(mut self, url: &str) -> Self {
        self.endpoint = url.to_string();
        self
    }

    /// Returns a list of namespaces the user has access to
    pub async fn namespaces(&self) -> Result<Vec<Namespace>, Error> {
        let res = self.get("/namespaces").send().await?;

        if res.status().is_success() {
            res.json::<NamespaceList>()
                .await
                .map(|list| list.namespaces)
                .map_err(Into::into)
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
    pub async fn images(&self) -> Result<Vec<Image>, Error> {
        let res = self.get("/images").send().await?;

        if res.status().is_success() {
            res.json::<ImageList>()
                .await
                .map(|list| list.images)
                .map_err(Into::into)
        } else {
            let err = res.json::<ErrorMessage>().await?;
            Err(Error::ApiError(err.message))
        }
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(&format!("{}{}", self.endpoint, path))
            .header("X-Auth-Token", &self.auth_token)
    }
}