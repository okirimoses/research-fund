# Decentralized Research Funding Platform

## Overview

This is a blockchain-based research funding platform built on the Internet Computer (IC) using Rust. The platform enables researchers to create profiles, submit research proposals, receive reviews, secure funding, and track milestones through a transparent and decentralized system.

## Features

### Researcher Management
- Create researcher profiles
- Unique identification with validation
- Track reputation scores and achievements

### Research Proposals
- Submit detailed research proposals
- Set funding targets
- Track proposal stages
- Review and fund proposals

### Milestone Tracking
- Define project milestones
- Submit and verify milestone proofs
- Monitor funding progress

### Review System
- Peer review mechanism
- Stake-based review process
- Verification of reviews

## Data Structures

### Key Entities
- `Researcher`: Represents a researcher's profile
- `ResearchProposal`: Defines research project details
- `Milestone`: Tracks project milestones
- `Review`: Manages peer reviews
- `ProofOfReproduction`: Validates research methodology

## Technical Details

### Technology Stack
- Language: Rust
- Platform: Internet Computer
- Key Libraries:
  - `serde`: Serialization
  - `candid`: Candid interface
  - `ic-stable-structures`: Stable data storage

### Memory Management
- Uses `StableBTreeMap` for persistent storage
- Thread-local static variables for data management
- Unique ID generation mechanism

## Functions

### Researcher Functions
- `create_researcher`: Register new researcher
- `get_researcher_by_id`: Retrieve researcher details
- `get_researcher_by_owner`: Get researcher profile for current user

### Proposal Functions
- `create_proposal`: Submit new research proposal
- `get_proposal_by_id`: Fetch proposal details
- `fund_proposal`: Support research with funding

### Milestone Functions
- `create_milestone`: Define project milestone
- `verify_milestone`: Confirm milestone completion
- `submit_proof`: Provide reproduction evidence

### Review Functions
- `submit_review`: Add peer review
- `get_reviews_by_proposal_id`: Retrieve proposal reviews

## Validation Mechanisms

- Email format validation
- Phone number format validation
- Input length and content checks
- Duplicate prevention
- Ownership verification

## Requirements
* rustc 1.64 or higher
```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```
* rust wasm32-unknown-unknown targetz
```bash
$ rustup target add wasm32-unknown-unknown
```
* candid-extractor
```bash
$ cargo install candid-extractor
```
* install `dfx`
```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

Clone the repository:
```bash
git clonehttps://github.com/okirimoses/research-fund.git

```
Navigate to the repository folder

```bash
cd research-fund
```

## Update dependencies

update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:
```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## did autogenerate

Add this script to the root directory of the project:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

Update line 16 with the name of your canister:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

After this run this script to generate Candid.
Important note!

You should run this script each time you modify/add/remove exported functions of the canister.
Otherwise, you'll have to modify the candid file manually.

Also, you can add package json with this content:
```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

and use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```