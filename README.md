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

### Basic Usage

```bash
# Create a mint configuration
token-acl create-config <MINT_ADDRESS> --authority <AUTHORITY_KEYPAIR>

# Set a gate program
token-acl set-gating-program <MINT_ADDRESS> <GATE_PROGRAM_ID>

# Enable permissionless thaw
token-acl enable-permissionless-thaw <MINT_ADDRESS>
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
cargo test --package token-acl --test integration
```

## License

MIT License - see [LICENSE](LICENSE) file for details