## How to compute the test coverage (note to myself)

rustup component add llvm-tools-preview
cargo install grcov

cd hamgraph
set RUSTFLAGS=-Cinstrument-coverage
set LLVM_PROFILE_FILE=target/coverage/hamcov-%p-%m.profraw
cargo clean
cargo build
cargo test --tests

grcov target/coverage --binary-path ../target/debug -s . -o target/coverage --output-types html,cobertura

The 'hard' part is that, even if working in a workspace with several crates, I should have (wherever I am, here I set CWD to hamgraph, but...) : 
- That grcov target (first arg) pointing to myWS/hamgraph/target/coverage
  - Because those PROFRAW files were generated in the various crates' target/ folders
- Binary path pointing to the workspace binary target, here '../target/debug' 

Otherwise by pointing everything to myWS/target/coverage I didn't get anything collected for the hamgraph crate
