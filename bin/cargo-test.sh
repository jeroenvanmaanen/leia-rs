#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

COLOR='always'
if [[ ".$1" = '.--color' ]]
then
  COLOR="$2"
  shift 2
fi

mkdir -p "${PROJECT}/tmp"

(
    cd "${PROJECT}"
    export RUST_LOG='info,igor=debug'
    cargo --color "${COLOR}" test -- --nocapture "$@" 2>&1 | tee ~/tmp/cargo-test-leia.log
)
