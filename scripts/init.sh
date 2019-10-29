#!/usr/bin/env bash

set -e

echo "*** Initialising WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
      rustup update nightly
      rustup update stable
      rustup target add wasm32-unknown-unknown --toolchain nightly
      # Install wasm-gc. It's useful for stripping slimming down wasm binaries.
      command -v wasm-gc || \
            cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force
fi
