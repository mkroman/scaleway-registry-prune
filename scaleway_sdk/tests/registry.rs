mod support;
use support::*;

use scaleway_sdk::registry;

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
