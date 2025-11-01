# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-10-31
### Added
- Initial stable release of SHADE: Simple Host Attestation & Dynamic Enrollment.
- TCP Proxy to validate connecting IPs against registered keys.
- HTTP API for key registration, host registration, and configuration validation.
- CLI commands:
  - `shade server` - start the SHADE server
  - `shade gen-keys` - generate private/public keypairs
  - `shade register-key` - register keys with optional expiration
  - `shade register-host` - register a host by public key
  - `shade list-keys` - list all registered keys
  - `shade list-hosts` - list all registered hosts
  - `shade revoke-cert` - revoke a certificate by ID
  - `shade validate` - validate the configuration file
- Support for `x-forwarded-for` and `forwarded` headers for proxied clients.
- SQLite backend for persistent key and host storage.
- Pre-commit hooks for Rust formatting (`cargo fmt`), linting (`cargo clippy`), and tests.

### Fixed
- N/A (this is the inaugural release; all features are new)

### Notes
- The proxy only permits IPs with registered keys; all other connections are rejected.
- Keys can have expiration dates; expired keys are automatically rejected.
- Designed for both testing (`127.0.0.1`) and production usage via a configurable YAML file.

