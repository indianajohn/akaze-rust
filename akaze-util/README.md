# Utilities for A-KAZE Feature Detector, Extractor, and Matcher for Rust

## Running Demonstrations
Note: These demonstrations refer to the [the akaze crate](../README.md).

```bash
# All executables (and your code probably) should be run in release mode, otherwise
# these can be quite slow.
# Extraction
cargo run --release --bin extract_features -- test-data/2.jpg output.bin

# Matching
cargo run --release --bin extract_and_match -- -m matches.png test-data/1.jpg test-data/2.jpg testname

# Output visualizations of detected features and scale space to directory `visualization`.
cargo run --release --bin extract_features -- test-data/2.jpg output.bin -d visualization
```
