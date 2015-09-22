#!/usr/bin/env bash
set -x

BIN_DIR=$(cd `dirname "${BASH_SOURCE[0]}"` && pwd)

/$BIN_DIR/stop.sh
docker rm open-analytics-db
