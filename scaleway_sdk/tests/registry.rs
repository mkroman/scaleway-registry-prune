mod support;
use support::*;

use scaleway_sdk::registry::{self, Status};

fn new_registry(endpoint: &str) -> registry::Registry {
    registry::Registry::new("token".to_owned(), "region".to_owned()).endpoint(endpoint)
}

#[tokio::test]
async fn it_parses_namespace_list() {
    let server = server::http(move |req| async move {
        assert_eq!(req.uri(), "/namespaces");

        http::Response::builder()
            .body(include_str!("fixtures/namespace_list.json").into())
            .unwrap()
    });

    let endpoint = format!("http://{}", server.addr());
    let registry = new_registry(&endpoint);
    let namespaces = registry.namespaces().await.unwrap();

    assert_eq!(namespaces.len(), 1);
    assert_eq!(namespaces.first().unwrap().name(), "mynamespace");
}

#[tokio::test]
async fn it_parses_image_tag_list() {
    let server = server::http(move |req| async move {
        assert_eq!(
            req.uri(),
            "/images/b00f6b0a-cc14-4c21-843f-3acda6ebb001/tags"
        );

        http::Response::builder()
            .body(include_str!("fixtures/image_tag_list.json").into())
            .unwrap()
    });

    let endpoint = format!("http://{}", server.addr());
    let registry = new_registry(&endpoint);
    let image_tags = registry
        .image_tags("b00f6b0a-cc14-4c21-843f-3acda6ebb001")
        .await
        .unwrap();

    assert_eq!(image_tags.len(), 27);
    assert_eq!(image_tags.first().unwrap().name(), "latest");
    assert_eq!(image_tags.first().unwrap().status(), Status::Ready);
}
