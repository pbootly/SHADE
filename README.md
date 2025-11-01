# SHADE
_Simple Host Attestation & Dynamic Enrollment_

SHADE is a generalized proxy for protecting services via simple node attestation using IP addresses. It is managed with a CLI tool to create, add, revoke, and validate certificates.

---

## ⚡ Features

- Generate and manage client keypairs  
- Register keys with optional expiration  
- Revoke keys or certificates  
- Store and validate edge node IPs  
- Transparent TCP proxy with attestation  

---
```
+-----------------+                                             
|   Client/Edge   |                                             
|   Node          |────────────────────────────────────────────┐
+-----------------+                                            │
          |                                                    │
          |  Register public key                               │
          v                                                    │
+-----------------+                                            │
| SHADE HTTP      |                                            │
| Server          |                                            │
| (Registration & |                                            │
|  Key Storage)   |                                            │
+-----------------+                                            │
          |                                                    │
          |  Stores client IP upon successful registration     │
          v                                                    │
+-----------------+                                            │
| SHADE TCP       |                                            │
| Proxy           |                                            │
| (IP Validation) |◄────────────────IP─────────────────────────┘
+-----------------+                                             
          |                                                     
          |  Allows traffic for registered IP                   
          v                                                     
+-----------------+                                             
| Protected       |                                             
| Service         |                                             
+-----------------+

```

##  Installation & Usage

### Start the server

By default, `shade server` runs for testing on `127.0.0.1` using the default configuration:

```sh
shade server
```

For production - specify a configuration file with the `-c` flag:

```sh
shade -c example_config.yaml server
```

### Key registration
Generate a client keypair (with access to shade socket):

```sh
shade gen-keys
```

Register the keypair (with access to shade socket):

```sh
shade register-key --private-key "K4H8FURo0WnWM24y3I5sSN+0aECmS1CceK2i8PACeyE="
```

Optionally, add expiration date:

```sh
shade register-key --private-key "K4H8FURo0WnWM24y3I5sSN+0aECmS1CceK2i8PACeyE=" --expires-at "2025-12-31T23:59:59Z"
```

### Host registration
On an edge node - register the host
```sh
shade register-host --public-key "hUQ1JHW1noXPZKXHidDgikT4iWC1/wEj+LR8gAPYGgE="
```

### Administrative commands

* List registered certificates
```sh
shade list-keys
```

* Revoke a certificate
```sh
shade revoke-cert --id "<UUID>"
```

* Validate configuration
```sh
shade validate
```

### E2E demo (`e2e.sh`)
```bash
#!/usr/bin/env bash
set -euo pipefail

fail() { echo "$1"; exit 1; }

# Build the SHADE binary
cargo build || fail "Build failed"

SHADE="target/debug/shade -c ./example_config.yaml"

# Generate a keypair
keys=$($SHADE gen-keys)
public_key=$(echo "$keys" | jq -r .public)
private_key=$(echo "$keys" | jq -r .private)

# Register the private key
$SHADE register-key --private-key "$private_key"

# List keys
$SHADE list-keys

# Register host
$SHADE register-host --public-key "$public_key" --url "http://localhost:3000"

# List hosts
$SHADE list-hosts
```


