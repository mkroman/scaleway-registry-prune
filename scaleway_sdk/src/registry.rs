use std::cmp::Ordering;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize};

pub use crate::status::Status;
use crate::Error;

static DEFAULT_API_ENDPOINT: &str = "https://api.scaleway.com/registry/v1";

pub struct Registry {
    client: reqwest::Client,
    region: String,
    endpoint: String,
    auth_token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Namespace {
    id: String,
    name: String,
    size: Option<usize>,
    description: String,
    organization_id: String,
    #[serde(deserialize_with = "Status::deserialize")]
    status: Status,
    status_message: String,
    endpoint: String,
    is_public: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    image_count: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Image {
    id: String,
    name: String,
    namespace_id: String,
    #[serde(deserialize_with = "Status::deserialize")]
    status: Status,
    status_message: Option<String>,
    visibility: String,
    size: usize,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    tags: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, Eq)]
pub struct ImageTag {
    id: String,
    name: String,
    image_id: String,
    #[serde(deserialize_with = "Status::deserialize")]
    status: Status,
    digest: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ImageTag {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn image_id(&self) -> &str {
        &self.image_id
    }

    pub fn status(&self) -> Status {
        self.status.clone()
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Returns created_at
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns updated_at
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Returns true if the given `date_time` is older than the last time this image tag was
    /// updated
    pub fn is_older_than(&self, date_time: DateTime<Utc>) -> bool {
        self.updated_at < date_time
    }

    /// Returns true if the given `date_time` is newer than the last time this
    /// tag was updated
    pub fn is_newer_than(&self, date_time: DateTime<Utc>) -> bool {
        self.updated_at >= date_time
    }
}

impl Ord for ImageTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.updated_at.cmp(&other.updated_at)
    }
}

impl PartialOrd for ImageTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ImageTag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Deserialize, Debug)]
struct NamespaceListResponse {
    namespaces: Vec<Namespace>,
    total_count: usize,
}

#[derive(Deserialize, Debug)]
struct ImageListResponse {
    images: Vec<Image>,
    total_count: usize,
}

#[derive(Deserialize, Debug)]
struct ImageTagListResponse {
    tags: Vec<ImageTag>,
    total_count: usize,
}

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
    pub fn size(&self) -> Option<usize> {
        self.size
    }

    /// Returns the user-defined description of the namespace
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the unique organization id
    pub fn organization_id(&self) -> &str {
        &self.organization_id
    }

    /// Returns the namespace status
    pub fn status(&self) -> Status {
        self.status.clone()
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
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns updated_at
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Returns image_count
    pub fn image_count(&self) -> usize {
        self.image_count
    }
}

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
    pub fn status(&self) -> Status {
        self.status.clone()
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
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns updated_at
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
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
    /// Creates a new `Registry` API instance
    pub fn new(auth_token: String, region: String) -> Self {
        let client = reqwest::ClientBuilder::new()
            .timeout(StdDuration::from_secs(30))
            .build()
            .unwrap();

        Registry {
            client,
            endpoint: format!("{}/regions/{}", DEFAULT_API_ENDPOINT, region),
            auth_token,
            region,
        }
    }

    /// Sets endpoint `url` by mutating self
    pub fn endpoint(mut self, url: &str) -> Self {
        self.endpoint = url.to_string();
        self
    }

    /// Returns a list of namespaces the user has access to
    pub async fn namespaces(&self) -> Result<Vec<Namespace>, Error> {
        // FIXME: Implement proper page handling
        self.get_deserialized::<NamespaceListResponse>("/namespaces")
            .await
            .map(|x| x.namespaces)
    }

    /// Returns the namespace details for a given `namespace_id`
    pub async fn namespace(&self, namespace_id: &str) -> Result<Namespace, Error> {
        self.get_deserialized::<Namespace>(&format!("/namespaces/{}", namespace_id))
            .await
    }

    /// Returns a list of all images accessible to the user
    pub async fn images(&self) -> Result<Vec<Image>, Error> {
        // FIXME: Implement proper page handling
        self.get_deserialized::<ImageListResponse>("/images")
            .await
            .map(|x| x.images)
    }

    /// Retrieves all tags for a given `image` and returns them
    pub async fn image_tags(&self, image_id: &str) -> Result<Vec<ImageTag>, Error> {
        // FIXME: Implement proper page handling
        let res = self
            .get(&format!("/images/{}/tags", image_id))
            .query(&[("page_size", "100")])
            .send()
            .await?;

        if res.status().is_success() {
            res.json::<ImageTagListResponse>()
                .await
                .map_err(Into::into)
                .map(|x| x.tags)
        } else {
            let err = res.json::<ErrorMessage>().await?;

            Err(Error::ApiError(err.message))
        }
    }

    /// Deletes an image with the given `image_tag` if it exists - the operation will fail if two
    /// tags share the same digest unless `force` is true
    pub async fn delete_image_by_tag(&self, tag_id: &str, force: bool) -> Result<ImageTag, Error> {
        // FIXME: deal with force properly
        let mut req = self.delete(&format!("/tags/{}", tag_id));

        if force {
            req = req.query(&[("force", "true")]);
        }

        let res = req.send().await?;

        if res.status().is_success() {
            res.json::<ImageTag>().await.map_err(Into::into)
        } else {
            let err = res.json::<ErrorMessage>().await?;

            Err(Error::ApiError(err.message))
        }
    }

    /// Requests the given `path` on the API endpoint and tries to deserialize
    /// it as json into the type `D`.
    async fn get_deserialized<D: DeserializeOwned>(&self, path: &str) -> Result<D, Error> {
        let res = self.get(path).send().await?;

        if res.status().is_success() {
            res.json::<D>().await.map_err(Into::into)
        } else {
            let err = res.json::<ErrorMessage>().await?;

            Err(Error::ApiError(err.message))
        }
    }

    /// Returns a prepared `RequestBuilder` that is ready to issue a GET request
    /// to the given `path` and a `X-Auth-Token` header already set
    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(&format!("{}{}", self.endpoint, path))
            .header("X-Auth-Token", &self.auth_token)
    }

    /// Returns a prepared `RequestBuilder` that is ready to issue a DELETE request
    /// to the given `path` and a `X-Auth-Token` header already set
    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .delete(&format!("{}{}", self.endpoint, path))
            .header("X-Auth-Token", &self.auth_token)
    }
}
