extern crate akaze;
use std::path::Path;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate image;
extern crate serde;
extern crate serde_json;
use akaze::types::evolution::{write_evolutions, Config};
use akaze::types::keypoint::{draw_keypoints_to_image};
use clap::{App, Arg};
use std::fs::File;
use std::io::{Read, Write};
use std::time::SystemTime;
use akaze_util::*;

fn main() {
    let matches = App::new("KAZE extractor.")
        .version("0.1")
        .about(
            "A Rust implementation of the KAZE visual feature extractor. See 
       https://github.com/pablofdezalc/kaze for the original authors' project. 
       Set AKAZE_LOG to debug for more verbose output.",
        )
        .author("John Stalbaum")
        .arg(
            Arg::with_name("INPUT")
                .help("The input image.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("The output extractions. Extension can be JSON or CBOR.")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("debug_path")
                .short("d")
                .long("debug_path")
                .value_name("DIRECTORY")
                .help("Sets a directory to write debug information to.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("options")
                .short("o")
                .long("options")
                .value_name("PATH")
                .help("A JSON file containing options.")
                .takes_value(true),
        )
        .get_matches();

    let start = SystemTime::now();
    let env = env_logger::Env::default().filter_or("AKAZE_LOG", "info");
    env_logger::Builder::from_env(env).init();
    let input_path = matches.value_of("INPUT").unwrap();
    let output_path = matches.value_of("OUTPUT").unwrap();
    info!(
        "Input image path is {}, output extractions path is {}.",
        input_path, output_path
    );
    let mut options = Config::default();
    match matches.value_of("options") {
        Some(options_path) => {
            if Path::new(options_path).exists() {
                info!("Reading options file from {}", options_path);
                let mut file = File::open(options_path).unwrap();
                let mut buffer = String::new();
                file.read_to_string(&mut buffer).unwrap();
                options = serde_json::from_str(&buffer).unwrap();
            } else {
                let mut file = File::create(options_path).unwrap();
                let serialized = serde_json::to_string(&options).unwrap();
                file.write_all(serialized.as_bytes()).unwrap();
                info!("Writing options file from {}", options_path);
            }
        }
        None => debug!("Using default options."),
    }
    let (evolutions, keypoints, descriptors) =
        akaze::extract_features(Path::new(input_path).to_owned(), options);
    let features = Features { keypoints, descriptors };
    serialize_features_to_file(&features, output_path).expect("failed to write out features");
    info!("Done, extracted {} features.", features.keypoints.len());
    match matches.value_of("debug_path") {
        Some(val) => {
            info!("Writing scale space since --debug_path/-d option was specified.");
            let string_to_pass = val.to_string();
            let path_to_scale_space_dir = std::path::Path::new(&string_to_pass.clone()).to_owned();
            std::fs::create_dir_all(&string_to_pass.clone()).unwrap();
            write_evolutions(&evolutions, path_to_scale_space_dir.clone());
            let mut input_image = image::open(Path::new(input_path).to_owned())
                .unwrap()
                .to_rgb();
            draw_keypoints_to_image(&mut input_image, &features.keypoints);
            let mut path_to_keypoint_image = path_to_scale_space_dir.clone();
            path_to_keypoint_image.push("keypoints.png");
            match input_image.save(path_to_keypoint_image.to_owned()) {
                Ok(_val) => debug!("Wrote keypoint image successfully."),
                Err(_e) => debug!("Could not write keypoint image for some reason, skipping."),
            }
        }
        None => {
            debug!("Argument --debug_path/-d was not given, not writing debug directory.");
        }
    }
    debug!("Total duration: {:?}", start.elapsed().unwrap());
}
