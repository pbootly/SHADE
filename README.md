# SHADE
_Simple Host Attestation &amp; Dynamic Enrollment_

SHADE aims to be a generalised proxy for protecting services with relatively trivial node attestment via IP address. It is managed with a CLI tool to create, add and revoke certificates.

## Usage:

### Start the server
```bash
./shade server
```

### Generate client keypair
```bash
./shade  gen-keys
Private key: K4H8FURo0WnWM24y3I5sSN+0aECmS1CceK2i8PACeyE=
Public key:  hUQ1JHW1noXPZKXHidDgikT4iWC1/wEj+LR8gAPYGgE=
```

This will output a generated private and public keypair. Use the output for the next steps.

# TODO: 
- [ ] Register generated keypair
```sh
shade register-keys --public="hUQ1JHW1noXPZKXHidDgikT4iWC1/wEj+LR8gAPYGgE=" --private="K4H8FURo0WnWM24y3I5sSN+0aECmS1CceK2i8PACeyE="
```

- [ ] List registered certificates
```sh
shade list-certs
```

- [ ] Revoke a certificate by public key 
```sh
shade revoke-cert --public="hUQ1JHW1noXPZKXHidDgikT4iWC1/wEj+LR8gAPYGgE="
```

- [ ] Provide small client binary that could be placed on a host
```sh
shade register --join-cert"hUQ1JHW1noXPZKXHidDgikT4iWC1/wEj+LR8gAPYGgE=" --host="localhost:5000/register"
```

