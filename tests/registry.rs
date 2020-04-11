mod support;
use support::*;

use scaleway_registry_prune::scaleway_sdk::registry;

#[tokio::test]
async fn it_parses_namespace_list() {
    let server = server::http(move |req| async move {
        assert_eq!(req.uri(), "/namespaces");

        http::Response::builder()
            .body(include_str!("fixtures/namespace_list.json").into())
            .unwrap()
    });

    let endpoint = format!("http://{}", server.addr());
    let registry =
        registry::Registry::new("my_token".to_owned(), "my_region".to_owned()).endpoint(&endpoint);

    let namespaces = registry.namespaces().await.unwrap();

    assert_eq!(namespaces.len(), 1);
    assert_eq!(namespaces.first().unwrap().name(), "mynamespace");
}
