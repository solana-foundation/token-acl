# Token ACL

A Solana program implementation of [SRFC37](srfc37.md) - a permissioned token standard that enables allow/block listing without compromising user experience or developer composability.

## Overview

Token ACL provides a novel mechanism for permissioned tokens using Token22's Default Account State extension and a delegated freeze authority. This approach eliminates the UX friction of manual token account thawing while maintaining protocol composability.

## Architecture

- **Token ACL Program** (`/program`): Core smart contract that manages freeze authority delegation and permissionless thaw/freeze operations
- **Interface Package** (`/interface`): Defines instruction discriminators and provides on/offchain account resolution for gate program implementers
- **Client Libraries** (`/clients`): SDKs for TypeScript, Rust, and CLI interaction
- **Examples** (`/examples`): Reference implementations of gate programs for testing

## Key Features

- **Managed Freeze Authority**: Token ACL manages the mint freeze authority 
- **Permissionless Thaw/Freeze**: Users can thaw/freeze token accounts without issuer intervention
- **Gate Program Interface**: Standardized interface for custom allow/block list logic
- **Composability**: Works with existing protocols without requiring specialized UIs
- **Security**: De-escalated permissions prevent malicious instruction injection

## Quick Start

### Prerequisites

- Rust 1.70+
- Solana CLI 2.2.0+
- Node.js 18+ (for TypeScript client)

### Installation

```bash
# Clone the repository
git clone https://github.com/solana-foundation/token-acl.git
cd token-acl

# Build all components
cargo build --release

# Install CLI
cargo install --path clients/cli
```

Installing the CLI from crates.io

```bash
cargo install token-acl-cli
```

### CLI Usage

#### Configuration Commands

```bash
# Create a mint configuration (transfers freeze authority to TokenACL program)
token-acl-cli create-config <MINT_ADDRESS> [--gating-program <GATING_PROGRAM>]

# Delete a mint configuration
token-acl-cli delete-config <MINT_ADDRESS> [--receiver <RECEIVER_ADDRESS>]

# Set the authority of a mint config
token-acl-cli set-authority <MINT_ADDRESS> --new-authority <NEW_AUTHORITY>

# Set the gating program for a mint config
token-acl-cli set-gating-program <MINT_ADDRESS> <NEW_GATING_PROGRAM>

# Enable/disable permissionless instructions
token-acl-cli set-instructions <MINT_ADDRESS> --enable-thaw --enable-freeze
token-acl-cli set-instructions <MINT_ADDRESS> --disable-thaw --disable-freeze
```

#### Freeze/Thaw Commands

```bash
# Freeze a token account (requires freeze authority)
token-acl-cli freeze <TOKEN_ACCOUNT>

# Thaw a token account (requires freeze authority)
token-acl-cli thaw <TOKEN_ACCOUNT>

# Freeze a token account permissionlessly
token-acl-cli freeze-permissionless --token-account <TOKEN_ACCOUNT>
# OR
token-acl-cli freeze-permissionless --mint <MINT_ADDRESS> --owner <TOKEN_ACCOUNT_OWNER>

# Thaw a token account permissionlessly
token-acl-cli thaw-permissionless --token-account <TOKEN_ACCOUNT>
# OR
token-acl-cli thaw-permissionless --mint <MINT_ADDRESS> --owner <TOKEN_ACCOUNT_OWNER>

# Create an associated token account and thaw it permissionlessly
token-acl-cli create-ata-and-thaw-permissionless --mint <MINT_ADDRESS> --owner <TOKEN_ACCOUNT_OWNER>
```

## Examples

- `token-acl-gate`: Gate program that enables the creation of allow and/or block lists. 
Full fledged implementation that allows combining multiple list checks and special handling for non-PDA (EoA) wallets.
Implementation [here](https://github.com/solana-foundation/token-acl-gate).

The [examples](examples) folder contains toy programs for testing purposes:

- `always-allow`: Always permits thaw/freeze operations
- `always-block`: Always blocks thaw/freeze operations  
- `always-allow-with-deps`: Example with additional account dependencies

## Specification

This implementation follows [sRFC37 - Token ACL](srfc37.md) which defines:

- Token ACL program interface and account structures
- Gate program interface with standardized discriminators
- Extra account metas resolution for dynamic account dependencies
- Security model with de-escalated permissions

## Development

```bash
# Run tests
cargo test

# Build program for deployment
cargo build-sbf --manifest-path program/Cargo.toml

# Run integration tests
cargo test --package token-acl-client
```

## License

MIT License - see [LICENSE](LICENSE) file for details