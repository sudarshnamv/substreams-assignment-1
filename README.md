# Solana Transfer Extraction using Substreams

## Overview
This project uses Rust and Substreams to extract native SOL transfer instructions from Solana blocks.

## Block Range
- Start block: 385870151
- End block: 385870156

## Output
The extractor produces transfer records containing:
- `from`: Sender wallet address in Base58
- `to`: Receiver wallet address in Base58
- `amount`: Transfer amount in lamports

## Implementation
- Rust-based Substreams map module
- Processes top-level and inner instructions
- Filters failed transactions
- Identifies native SOL System Program transfer instructions
- Encodes wallet addresses using Base58

## Build
```bash
substreams build

. ./.substreams.env
substreams run -e mainnet.sol.streamingfast.io:443 -s 385870151 -t 385870157 ./substreams.yaml map_transfers -o jsonl
