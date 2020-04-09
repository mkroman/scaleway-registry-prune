use clap::{crate_authors, crate_name, crate_version, App, Arg};
mod scaleway;

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

fn validate_image_arg(arg: String) -> Result<(), String> {
    parse_image_argument(&arg)
        .ok_or("Must be specified in the format `<namespace>/<image>'".to_owned())
        .map(|_| ())
}

fn main() {
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

    let scw_token = matches.value_of("token").expect("token missing");

    if let Some((namespace, image)) = parse_image_argument(matches.value_of("IMAGE").unwrap()) {
        println!("namespace: {} image: {}", namespace, image);
    }
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
