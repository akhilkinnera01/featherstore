#!/usr/bin/env bash
set -euo pipefail

SERVER_URL="${SERVER_URL:-http://localhost:8080}"

featherstore-load-synthetic   --server "$SERVER_URL"   --users 1000 --items 1000 --contexts 24   --embedding-dims 4 --batch-size 500

featherstore-benchmark   --server "$SERVER_URL"   --scenario single_get_request_qps   --duration 5 --warmup 1 --concurrency 16 --target-qps 1000   --users 1000 --items 1000 --contexts 24 --embedding-dims 4   --output benchmark-smoke.json

printf 'wrote benchmark-smoke.json
'
