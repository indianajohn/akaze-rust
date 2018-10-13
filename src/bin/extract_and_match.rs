extern crate akaze;
use std::path::Path;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate image;
extern crate serde;
extern crate serde_json;
use akaze::types::keypoint::serialize_to_file;
use akaze::match_features;
use akaze::types::evolution::Config;
use akaze::types::feature_match;
use clap::{App, Arg};
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
        ).author("John Stalbaum")
        .arg(
            Arg::with_name("INPUT_0")
                .help("The first input image.")
                .required(true)
                .index(1),
        ).arg(
            Arg::with_name("INPUT_1")
                .help("The second input image.")
                .required(true)
                .index(2),
        ).arg(
            Arg::with_name("OUTPUT_PREFIX")
                .help("The output prefix for all files.")
                .required(true)
                .index(3),
        ).arg(
            Arg::with_name("match_image_path")
                .short("m")
                .long("match_image")
                .value_name("IMAGE_FILE_PATH")
                .help("Sets a path to write the match image to.")
                .takes_value(true),
        ).get_matches();

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
        "Input image paths are {}/{}, output extractions path is {}, threshols is {}.",
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
    let (_evolutions_0, keypoints_0, descriptors_0) = akaze::extract_features(
        Path::new(input_path_0).to_owned(),
        options,
    );
    serialize_to_file(&keypoints_0, &descriptors_0, Path::new(&extractions_0_path).to_owned());
    info!(
        "Done, extracted {} features from image 0.",
        keypoints_0.len()
    );
    let (_evolutions_1, keypoints_1, descriptors_1) = akaze::extract_features(
        Path::new(input_path_1).to_owned(),
        options,
    );
    serialize_to_file(&keypoints_0, &descriptors_0, Path::new(&extractions_1_path).to_owned());
    info!(
        "Done, extracted {} features from image 1, proceeding with matching.",
        keypoints_1.len()
    );
    let output_matches = match_features(&keypoints_0, &descriptors_0, &keypoints_1, &descriptors_1);
    info!("Got {} matches.", output_matches.len());
    feature_match::serialize_to_file(&output_matches, Path::new(&matches_path).to_owned());
    match matches.value_of("match_image_path") {
        Some(match_image_path) => {
            info!("Writing scale space since --debug_path/-d option was specified.");
            let mut input_image_0 = image::open(Path::new(input_path_0).to_owned())
                .unwrap()
                .to_rgb();
            let mut input_image_1 = image::open(Path::new(input_path_1).to_owned())
                .unwrap()
                .to_rgb();
            let matches_image = feature_match::draw_matches(
                &input_image_0,
                &input_image_1,
                &keypoints_0,
                &keypoints_1,
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
