use clap::{crate_authors, crate_name, crate_version, App, Arg};

use scaleway_sdk::{
    registry::{Image, Namespace, Registry},
    Error,
};

/// Takes a string in the format `<namespace>/<image>` and returns an Option
/// with a tuple in the format `(namespace, image)` unleess the input string is
/// malformed
fn parse_image_argument(arg: &str) -> Option<(&str, &str)> {
    let mut parts = arg.splitn(2, '/');

    match (parts.next(), parts.next()) {
        (Some(namespace), Some(image)) => {
            if namespace.is_empty() || image.is_empty() {
                None
            } else {
                Some((namespace, image))
            }
        }
        _ => None,
    }
}

/// Validates that the given `namespace/image` string can be parsed by `parse_image_argument`
///
/// This is used by `clap` when parsing arguments
fn validate_image_arg(arg: String) -> Result<(), String> {
    parse_image_argument(&arg)
        .ok_or_else(|| "Must be specified in the format `<namespace>/<image>'".to_owned())
        .map(|_| ())
}

/// Attempts to retrieve information about the given `image` and checks if it's
/// part of the given `namespace` before returning both, unless an error occurs
async fn get_image_info(
    registry: &Registry,
    namespace: &str,
    image: &str,
) -> Result<(Namespace, Image), Error> {
    let namespace_vec = registry.namespaces().await?;
    let namespace = namespace_vec
        .iter()
        .find(|ns| ns.name() == namespace)
        .ok_or_else(|| Error::NoSuchNamespace)?;

    let image_vec = registry.images().await?;

    let image = image_vec
        .iter()
        .filter(|x| x.namespace_id() == namespace.id())
        .find(|x| x.name() == image)
        .ok_or_else(|| Error::NoSuchImage)?;

    Ok((namespace.clone(), image.clone()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Prunes scaleway container registries")
        .arg(
            Arg::with_name("keep-within")
                .long("keep-within")
                .value_name("duration")
                .help(
                    "Keep versions that are newer than duration (e.g. 3d) relative to current time",
                ),
        )
        .arg(
            Arg::with_name("keep-last")
                .long("keep-last")
                .value_name("n")
                .help("Keep the last n versions"),
        )
        .arg(
            Arg::with_name("region")
                .long("region")
                .help("The target region")
                .env("SCW_REGION")
                .required(true),
        )
        .arg(
            Arg::with_name("token")
                .long("scw-token")
                .env("SCW_TOKEN")
                .help("Authentication token")
                .required(true),
        )
        .arg(
            Arg::with_name("IMAGE")
                .index(1)
                .required(true)
                .value_name("NAMESPACE/IMAGE")
                .validator(validate_image_arg),
        )
        .get_matches();

    let region = matches.value_of("region").expect("missing region");
    let scw_token = matches.value_of("token").expect("missing token");
    let registry = Registry::new(scw_token.to_owned(), region.to_owned());

    let (namespace_name, image_name) =
        parse_image_argument(matches.value_of("IMAGE").unwrap()).unwrap();

    let result = get_image_info(&registry, namespace_name, image_name).await;

    match result {
        Ok((_, image)) => {
            println!("Image info: {:?}", image);
        }
        Err(e) => {
            println!("Error when fetching image info: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_parses_image_argument() {
        assert!(parse_image_argument("mynamespace/myimage").is_some());
        assert!(parse_image_argument("mynamespace").is_none());
    }

    #[test]
    fn it_doesnt_parse_empty_namespace_or_image() {
        assert!(parse_image_argument("/myimage").is_none());
        assert!(parse_image_argument("mynamespace/").is_none());
    }

    #[test]
    fn it_parses_namespace_and_image() {
        let res = parse_image_argument("mynamespace/myimage");

        assert_eq!(res.unwrap().0, "mynamespace");
        assert_eq!(res.unwrap().1, "myimage");
    }
}
