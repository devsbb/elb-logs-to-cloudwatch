#!/usr/bin/env bash
set -ex
DESTINATION_FILE="$1"
VERSION="0.4.0"
URL="https://github.com/devsbb/elb-logs-to-cloudwatch/releases/download/v${VERSION}/elb-logs-to-cloudwatch-lambda-${VERSION}.zip"
curl -Lso "$DESTINATION_FILE" "$URL"
