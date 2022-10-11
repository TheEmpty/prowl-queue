#!/bin/bash

set -ex

cargo fmt
cargo clippy -- -D warnings
cargo build --release
# cargo test --release
cargo doc --release

if $(git diff --quiet) ; then
  cargo publish
  git push
else
  echo "Dirty git tree, please fix and re-run."
fi
