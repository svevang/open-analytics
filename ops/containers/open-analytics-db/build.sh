#!/usr/bin/env bash
set -x
echo "Run this from the cargo app root"

BIN_DIR=$(cd `dirname "${BASH_SOURCE[0]}"` && pwd)

docker build -t svevang/open-analytics-db $BIN_DIR/build
