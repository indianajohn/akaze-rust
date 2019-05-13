use akaze::types::feature_match::Match;
use akaze::types::keypoint::{Descriptor, Keypoint};
use failure::Error;
use log::*;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Features {
    pub keypoints: Vec<Keypoint>,
    pub descriptors: Vec<Descriptor>,
}

/// Serialize features to a file.
pub fn serialize_features_to_file(
    features: &Features,
    path: impl AsRef<Path>,
) -> Result<(), Error> {
    let path = path.as_ref();
    debug!("Writing features to {:?}", path);
    let file = File::create(path)?;
    let extension = path.extension().and_then(OsStr::to_str).unwrap_or("bin");
    match extension {
        "json" => serde_json::to_writer(file, features)?,
        _ => bincode::serialize_into(file, features)?,
    }
    Ok(())
}

/// Serialize features to a file.
pub fn deserialize_features_from_file(path: impl AsRef<Path>) -> Result<Features, Error> {
    let path = path.as_ref();
    debug!("Reading features from {:?}", path);
    let file = File::open(path)?;
    let extension = path.extension().and_then(OsStr::to_str).unwrap_or("bin");
    Ok(match extension {
        "json" => serde_json::from_reader(file)?,
        _ => bincode::deserialize_from(file)?,
    })
}

/// Serialize matches to a file.
pub fn serialize_matches_to_file(matches: &[Match], path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();
    debug!("Writing matches to {:?}", path);
    let file = File::create(path)?;
    let extension = path.extension().and_then(OsStr::to_str).unwrap_or("bin");
    match extension {
        "json" => serde_json::to_writer(file, matches)?,
        _ => bincode::serialize_into(file, matches)?,
    }
    Ok(())
}

/// Serialize matches to a file.
pub fn deserialize_matches_from_file(path: impl AsRef<Path>) -> Result<Vec<Match>, Error> {
    let path = path.as_ref();
    debug!("Reading matches to {:?}", path);
    let file = File::open(path)?;
    let extension = path.extension().and_then(OsStr::to_str).unwrap_or("bin");
    Ok(match extension {
        "json" => serde_json::from_reader(file)?,
        _ => bincode::deserialize_from(file)?,
    })
}
