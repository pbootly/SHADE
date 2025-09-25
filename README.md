# SHADE
_Simple Host Attestation & Dynamic Enrollment_

SHADE aims to be a generalised proxy for protecting services with relatively trivial node attestment via IP address. It is managed with a CLI tool to create, add and revoke certificates.

## Usage:

### Start the server

By default, the `shade server` command is for testing purposes only. It binds to `127.0.0.1` and uses the default configuration. For production, specify a configuration file with the `-c` flag (see examples below).

```bash
shade server
```
### Production Example

To run SHADE in a production environment, specify the configuration file with the `-c` flag. For example:

```bash
shade -c example_config.yaml server
```

### Generate client keypair
```bash
shade gen-keys
Private key: K4H8FURo0WnWM24y3I5sSN+0aECmS1CceK2i8PACeyE=
Public key:  hUQ1JHW1noXPZKXHidDgikT4iWC1/wEj+LR8gAPYGgE=
```

This will output a generated private and public keypair. Use the output for the next steps.

### Register a keypair
```bash
shade register-key --private-key "K4H8FURo0WnWM24y3I5sSN+0aECmS1CceK2i8PACeyE="
```

This registers the generated keypair into the SHADE system.
### Register a keypair with expiration
```bash
shade register-key --private-key "K4H8FURo0WnWM24y3I5sSN+0aECmS1CceK2i8PACeyE=" --expires-at "2025-12-31T23:59:59Z"
```

This registers the generated keypair into the SHADE system with an expiration date.

### List registered certificates
```bash
shade list-keys
```

This command lists all the registered certificates in the SHADE system.

### Revoke a certificate by ID
```bash
shade revoke-cert --id "<UUID>"
```

This revokes a certificate with the specified ID.

### Validate the configuration
```bash
shade validate
```

This validates the SHADE configuration file.

