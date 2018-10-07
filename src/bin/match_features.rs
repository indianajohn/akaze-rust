extern crate akaze;
use std::path::Path;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate image;
extern crate serde;
extern crate serde_json;
use akaze::ops::feature_matching::descriptor_match;
use akaze::types::feature_match;
use akaze::types::keypoint;
use clap::{App, Arg};
use std::time::SystemTime;

fn main() {
    let matches = App::new("Feature matching using Hamming distance for AKAZE features.")
        .version("0.1")
        .about(
            "A Rust implementation of the KAZE visual feature matching using
            Hamming distance for binary descriptors. For use with AKAZE.
       Set RUST_LOG to debug for more verbose output.",
        ).author("John Stalbaum")
        .arg(
            Arg::with_name("INPUT_EXTRACTIONS_0")
                .help("The input extraction results for image 0.")
                .required(true)
                .index(1),
        ).arg(
            Arg::with_name("INPUT_EXTRACTIONS_1")
                .help("The input extraction results for image 1.")
                .required(true)
                .index(2),
        ).arg(
            Arg::with_name("OUTPUT")
                .help("The output matches.")
                .required(true)
                .index(3),
        ).get_matches();

    let start = SystemTime::now();
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();
    let input_extractions_0_path = matches.value_of("input_extractions_0").unwrap();
    let input_extractions_1_path = matches.value_of("input_extractions_1").unwrap();
    let output_path = matches.value_of("OUTPUT").unwrap();
    info!(
        "Input extractions: {}/{}, output matches: {}.",
        input_extractions_0_path, input_extractions_1_path, output_path
    );
    let extractions_0 =
        keypoint::deserialize_from_file(Path::new(input_extractions_0_path).to_owned());
    let extractions_1 =
        keypoint::deserialize_from_file(Path::new(input_extractions_1_path).to_owned());
    let matches = descriptor_match(
        &extractions_0.keypoints,
        &extractions_0.descriptors,
        &extractions_1.keypoints,
        &extractions_1.descriptors,
    );
    feature_match::serialize_to_file(&matches, Path::new(output_path).to_owned());
    debug!(
        "Done, got {} matches, total duration: {:?}",
        matches.len(),
        start.elapsed().unwrap()
    );
}
