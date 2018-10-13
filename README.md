# A-KAZE Feature Detector, Extractor, and Matcher for Rust
This repository contains a Rust implementation of the A-KAZE algorithm. I would like to 
thank the original authors of the A-KAZE algorithm, who provided the 
implementation I used as a reference here:

[A-KAZE original authors' GitHub repository](https://github.com/pablofdezalc/akaze)

The work in this paper was first published in the following papers.

Fast Explicit Diffusion for Accelerated Features in Nonlinear Scale Spaces. Pablo F. Alcantarilla, J. Nuevo and Adrien Bartoli. In British Machine Vision Conference (BMVC), Bristol, UK, September 2013

KAZE Features. Pablo F. Alcantarilla, Adrien Bartoli and Andrew J. Davison. In European Conference on Computer Vision (ECCV), Fiorenze, Italy, October 2012

In case you found this elsewhere, this code lives here:
https://github.com/indianajohn/akaze-rust/

## Summary
The algorithm used here was heavily inspired by that repository, but differs in places. The
resulting implementation produces results very similar to the original code, albeit a bit
more slowly. The code in this crate is 100% `safe` Rust, as are most of the dependencies.

## Results
This crate can be used to extract keypoints and descriptors from an image using
a non-linear scale space.
![Keypoints](/test-data/keypoints-1.jpg "A-KAZE keypoints")
![Keypoints](/test-data/keypoints-2.jpg "A-KAZE keypoints")

It also includes a function to do matching with the 8-point algorithm.
![Matches](/test-data/match_image.jpg "A-AKAZE matches")

## Example
```rust
 extern crate akaze;
 use std::path::Path;
 let options = akaze::types::evolution::Config::default();
 let (_evolutions_0, keypoints_0, descriptors_0) =
     akaze::extract_features(
       Path::new("test-data/1.jpg").to_owned(),
       options);

 let (_evolutions_1, keypoints_1, descriptors_1) =
     akaze::extract_features(
       Path::new("test-data/1.jpg").to_owned(),
       options);
 let matches = akaze::match_features(&keypoints_0, &descriptors_0, &keypoints_1, &descriptors_1);
akaze::types::feature_match::serialize_to_file(&matches, Path::new("matches.cbor").to_owned());
println!("Got {} matches.", matches.len());
```

## Running Demonstrations
```
# Extraction
cargo run --release --bin extract_features -- ./test-data/2.jpg ./output.cbor 

# Matching
cargo run --release --bin extract_and_match -- -m ./match_image.png ./test-data/1.jpg ./test-data/2.jpg

# Visualizing scale space
cargo run --release --bin extract_features -- ./test-data/2.jpg ./output.cbor  -d ./scale-space/
```

## License
This code is released under the MIT license. See LICENSE.md for more details. You're free to
use this however you'd like.

## Disclaimer
I chose to do this project to learn Rust. It's entirely possible this code does not follow
what some would consider the best Rust style. Use at your own risk.