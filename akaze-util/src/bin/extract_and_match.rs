#[macro_use]
extern crate log;

use akaze::match_features;
use akaze::types::evolution::Config;
use akaze::types::feature_match;
use akaze_util::*;
use clap::{App, Arg};
use std::path::Path;
use std::time::SystemTime;

fn main() {
    let matches = App::new("Extract and match KAZE image features..")
        .version("0.1")
        .about(
            "A Rust implementation of the KAZE visual feature extractor and matching.
       See https://github.com/pablofdezalc/kaze for the original authors' project. 
       Set AKAZE_LOG to debug for more verbose output. This executable runs the entire
       pipeline end-to-end for two images. For more granular control, see the binaries
       extract_features and match_features.",
        )
        .author("John Stalbaum")
        .arg(
            Arg::with_name("INPUT_0")
                .help("The first input image.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("INPUT_1")
                .help("The second input image.")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("OUTPUT_PREFIX")
                .help("The output prefix for all files.")
                .required(true)
                .index(3),
        )
        .arg(
            Arg::with_name("match_image_path")
                .short("m")
                .long("match_image")
                .value_name("IMAGE_FILE_PATH")
                .help("Sets a path to write the match image to.")
                .takes_value(true),
        )
        .get_matches();

    let start = SystemTime::now();
    let env = env_logger::Env::default().filter_or("AKAZE_LOG", "info");
    env_logger::Builder::from_env(env).init();
    let input_path_0 = matches.value_of("INPUT_0").unwrap();
    let input_path_1 = matches.value_of("INPUT_1").unwrap();
    let output_prefix = matches.value_of("OUTPUT_PREFIX").unwrap();
    let threshold: f64 = matches
        .value_of("threshold")
        .unwrap_or("10")
        .parse()
        .unwrap();
    info!(
        "Input image paths are {}/{}, output extractions path is {}, threshold is {}.",
        input_path_0, input_path_1, output_prefix, threshold,
    );
    let options = Config::default();
    let prefix_string: String = output_prefix.to_owned();
    let mut extractions_0_path = prefix_string.clone();
    extractions_0_path.push_str("-extractions_0.cbor");
    let mut extractions_1_path = prefix_string.clone();
    extractions_1_path.push_str("-extractions_1.cbor");
    let mut matches_path = prefix_string.clone();
    matches_path.push_str("-matches.cbor");

    let (_, keypoints, descriptors) =
        akaze::extract_features(Path::new(input_path_0).to_owned(), options);
    let features_0 = Features {
        keypoints,
        descriptors,
    };
    serialize_features_to_file(&features_0, extractions_0_path)
        .expect("failed to write first image features to file");
    info!(
        "Done, extracted {} features from image 0.",
        features_0.keypoints.len()
    );

    let (_, keypoints, descriptors) =
        akaze::extract_features(Path::new(input_path_1).to_owned(), options);
    let features_1 = Features {
        keypoints,
        descriptors,
    };
    serialize_features_to_file(&features_1, extractions_1_path)
        .expect("failed to write second image features to file");
    info!(
        "Done, extracted {} features from image 1, proceeding with matching.",
        features_1.keypoints.len()
    );

    let output_matches = match_features(
        &features_0.keypoints,
        &features_0.descriptors,
        &features_1.keypoints,
        &features_1.descriptors,
        0.86,
        1000,
        3.0,
    );
    info!("Got {} matches.", output_matches.len());

    serialize_matches_to_file(&output_matches, matches_path)
        .expect("failed to write matches to file");
    match matches.value_of("match_image_path") {
        Some(match_image_path) => {
            info!("Writing scale space");
            let input_image_0 = image::open(input_path_0).unwrap().to_rgb();
            let input_image_1 = image::open(input_path_1).unwrap().to_rgb();
            let matches_image = feature_match::draw_matches(
                &input_image_0,
                &input_image_1,
                &features_0.keypoints,
                &features_1.keypoints,
                &output_matches,
            );
            match matches_image.save(match_image_path.to_owned()) {
                Ok(_val) => debug!("Wrote matches image successfully."),
                Err(_e) => debug!("Could not write matches image for some reason, skipping."),
            }
        }
        None => {
            debug!("Argument --match_image_path/-i was not given, not writing matches image.");
        }
    }
    debug!("Total duration: {:?}", start.elapsed().unwrap());
}
