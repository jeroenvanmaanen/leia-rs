#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

COLOR='always'
if [[ ".$1" == '.--color' ]]
then
  COLOR="$2"
  shift 2
fi

RUST_LOG='debug'
if [[ ".$1" == '.--trace' ]]
then
  RUST_LOG=trace
  shift
fi
export RUST_LOG

mkdir -p "${PROJECT}/tmp"

cargo --color "${COLOR}" test -- --test-threads 1 --nocapture "$@" 2>&1 | tee ~/tmp/cargo-test-leia.log
