#!/bin/sh
set -x

docker run  \
  --name open-analytics \
  --link open-analytics-db:db \
  -e DB_URL=postgres://postgres@db/open_analytics \
  -d -p 3000:3000 \
  svevang/open-analytics
