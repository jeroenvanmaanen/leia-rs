#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"

RUST_LOG='info'
if [[ ".$1" == '.--debug' ]]
then
  RUST_LOG=debug
  shift
elif [[ ".$1" == '.--trace' ]]
then
  RUST_LOG=trace
  shift
fi

export RUST_LOG
export RUST_BACKTRACE=1
cargo run -- "$@"
