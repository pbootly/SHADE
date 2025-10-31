#!/usr/bin/env bash

set -euo pipefail

fail() {
  echo "$1"
  exit 1
}

# Build the shade binary
cargo build || fail "build failed"

SHADE="target/debug/shade -c ./example_config.yaml"

# Generate a keypair
keys=$($SHADE gen-keys)

public_key=$(echo "$keys" | jq -r .public)
private_key=$(echo "$keys" | jq -r .private)

# Register the private key
$SHADE register-key --private-key "$private_key"

# List keys
$SHADE list-keys

# register host
$SHADE register-host --public-key "$public_key" --url "http://localhost:3000"

# List host
$SHADE list-hosts
