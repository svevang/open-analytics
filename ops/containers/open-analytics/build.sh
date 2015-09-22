#!/usr/bin/env bash
set -x

BIN_DIR=$(cd `dirname "${BASH_SOURCE[0]}"` && pwd)

docker build -t svevang/open-analytics $BIN_DIR/../../../
