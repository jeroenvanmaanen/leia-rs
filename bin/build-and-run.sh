#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

(
    cd "${PROJECT}"
    export RUST_LOG='info'
    export RUST_BACKTRACE=1
    cargo run
)
