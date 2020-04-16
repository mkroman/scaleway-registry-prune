use std::fmt::Display;
use std::io::{self, Write};
use std::str::FromStr;

use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches};
use indicatif::{ProgressBar, ProgressStyle};

use scaleway_sdk::{
    registry::{Image, ImageTag, Namespace},
    Registry,
};

mod error;
use error::Error;

#[derive(Default)]
struct FilterOptions {
    keep_last: Option<u64>,
}

struct Options {
    token: String,
    region: String,
    image: String,
    namespace: String,
    filter: FilterOptions,
}

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

fn validate_parsable<T>(arg: String) -> Result<(), String>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    arg.as_str()
        .parse::<T>()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Attempts to retrieve information about the given `image` and checks if it's
/// part of the given `namespace` before returning both, unless an error occurs
async fn get_namespace_and_image(
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

/// Parses the `args` and returns an `Options` struct with the relevant fields set based on the
/// given args
fn parse_args(args: ArgMatches) -> Options {
    let (namespace, image) = parse_image_argument(args.value_of("IMAGE").unwrap()).unwrap();

    let keep_last = args
        .value_of("keep-last")
        .map(|s| s.parse::<u64>().unwrap());

    let filter = FilterOptions { keep_last };

    Options {
        region: args.value_of("region").expect("missing region").to_string(),
        token: args.value_of("token").expect("missing token").to_string(),
        image: image.to_string(),
        namespace: namespace.to_string(),
        filter,
    }
}

fn filter_image_tags<'a>(options: &Options, image_tags: &'a [ImageTag]) -> Vec<&'a ImageTag> {
    let filter = &options.filter;

    image_tags
        .iter()
        .enumerate()
        .filter(|&(i, _x)| {
            if let Some(n) = filter.keep_last {
                (i as u64) > n
            } else {
                true
            }
        })
        .map(|(_, x)| x)
        .collect::<Vec<&ImageTag>>()
}

fn read_answer_from_stdin() -> io::Result<String> {
    let mut answer = String::new();

    io::stdin().read_line(&mut answer)?;
    answer = answer.trim().to_string();

    Ok(answer)
}

async fn try_main() -> Result<(), Error> {
    env_logger::init();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Prunes scaleway container registries")
        .arg(
            Arg::with_name("keep-last")
                .help("Keep the last n versions")
                .long("keep-last")
                .validator(validate_parsable::<u64>)
                .value_name("n"),
        )
        .arg(
            Arg::with_name("region")
                .env("SCW_REGION")
                .help("The target region")
                .required(true)
                .long("region"),
        )
        .arg(
            Arg::with_name("token")
                .env("SCW_TOKEN")
                .help("Authentication token")
                .long("scw-token")
                .required(true),
        )
        .arg(
            Arg::with_name("IMAGE")
                .index(1)
                .required(true)
                .validator(validate_image_arg)
                .value_name("NAMESPACE/IMAGE"),
        )
        .get_matches();

    let options = parse_args(matches);
    let registry = Registry::new(options.token.clone(), options.region.clone());

    // Find the image by its provided name, then verify that it's in the correct namespace,
    // otherwise return an error
    let (_, image) = get_namespace_and_image(&registry, &options.namespace, &options.image).await?;

    // Get all tags for the image
    let mut tags = registry.image_tags(image.id()).await?;

    if tags.is_empty() {
        return Err(Error::NoImageTagsError);
    }

    tags.sort_by(|a, b| a.updated_at().cmp(&b.updated_at()));
    tags.reverse();

    let filtered_tags = filter_image_tags(&options, &tags);

    if filtered_tags.is_empty() {
        return Err(Error::NoMatchingImageTagsError);
    }

    println!("This will delete the following images:");

    for t in filtered_tags.iter() {
        println!("{}:{}\t{}", image.name(), t.name(), t.updated_at());
    }

    print!("Do you want to continue? [y/N] ");
    io::stdout().flush().unwrap();

    if let Ok(answer) = read_answer_from_stdin() {
        if answer == "y" || answer == "Y" {
            let ps = ProgressStyle::default_bar().template("{prefix} {wide_bar} {pos}/{len}");
            let pb = ProgressBar::new(filtered_tags.len() as u64).with_style(ps);

            for tag in filtered_tags.iter() {
                pb.set_prefix(&format!("{}:{}", image.name(), tag.name()));
                registry.delete_image_by_tag(tag.id(), false).await?;
                pb.inc(1);
            }

            pb.finish();
        }
    }

    Ok(())
}

fn main() {
    use tokio::runtime::Runtime;
    let mut rt = Runtime::new().expect("unable to create async runtime");

    match rt.block_on(try_main()) {
        Ok(_) => {}
        Err(e) => {
            println!("There was an error: {}", e);
        }
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
