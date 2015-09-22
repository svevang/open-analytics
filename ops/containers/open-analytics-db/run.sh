#!/bin/sh
set -x

docker run --name open-analytics-db \
  -d svevang/open-analytics-db
