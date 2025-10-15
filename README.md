# Dark Pool Matching Engine

A privacy-preserving orderbook and matching engine built on Solana using Arcium's Multi-Party Computation (MPC) network. This dark pool enables users to submit encrypted buy and sell orders without revealing amounts or prices publicly, with order matching performed confidentially off-chain and settlement executed on-chain.

## Overview

Traditional on-chain orderbooks expose all order details publicly, allowing front-running and information leakage. This dark pool solves that problem by encrypting sensitive order data (amounts and prices) and leveraging Arcium's MPC network to perform orderbook operations and matching confidentially. Only the execution results are revealed to the matched parties during settlement.

## Key Features

**Privacy-Preserving Orders**
- Order amounts and prices are encrypted using x25519 key exchange and RescueCipher
- Only the MPC network can decrypt and process orderbook operations
- Order IDs, user addresses, and statuses remain public for UX purposes

**Confidential Order Matching**
- Orderbook maintained as encrypted state (1302-byte ciphertext)
- Buy orders organized in max-heap (highest price first)
- Sell orders organized in min-heap (lowest price first)
- Price-time priority matching executed by MPC network
- Up to 10 orders per side, 5 matches per batch

**Secure Settlement**
- Match results encrypted for backend decryption only
- Backend derives vault addresses and executes token transfers
- SPL token-based transfers with locked fund management
- Settlement history recorded on-chain

**Nonce-Based Security**
- Every MPC operation increments an on-chain nonce
- Prevents replay attacks and ensures operation ordering
- Separate nonces for orderbook state and match results

## Architecture

The project consists of two main components:

### 1. Encrypted Instructions (`encrypted-ixs/`)
Confidential computation circuits written in Arcis (Arcium's Rust framework):
- `submit_order` - Adds encrypted orders to the orderbook
- `match_orders` - Finds crossing orders and generates matches

These circuits execute within Arcium's MPC network where sensitive data remains encrypted throughout computation.

### 2. Solana Program (`programs/matching_engine/`)
On-chain program that manages state and orchestrates MPC operations:
- **Initialization** - Set up program with backend authority and token pair
- **Vault Management** - User token deposits and withdrawals
- **Order Submission** - Queue MPC computation to add orders
- **Matching Trigger** - Initiate confidential matching process
- **Settlement** - Execute token transfers for matched orders

## Workflow

### Order Submission
1. User creates encrypted order (amount, price) using x25519 + RescueCipher
2. Program queues MPC computation with encrypted data
3. MPC network adds order to encrypted orderbook
4. Callback updates on-chain state and nonce
5. OrderAccount created with status and locked funds

### Order Matching
1. Backend triggers matching computation (rate-limited to 15s intervals)
2. MPC network decrypts orderbook, finds price crossings
3. Generates up to 5 matches with execution prices
4. Encrypts match results for backend (Enc<Shared, MatchResult>)
5. Callback emits MatchResultEvent with encrypted matches

### Settlement
1. Backend decrypts match results using match nonce
2. Derives buyer/seller vault PDAs from user pubkeys
3. Executes settlement instruction with match details
4. Program transfers tokens between vaults
5. Updates order statuses and vault balances

## Prerequisites

- **Rust** 1.75+ with Solana toolchain
- **Solana CLI** 1.18+
- **Anchor Framework** 0.31.1
- **Arcium CLI** - For MPC network interaction
- **Node.js** 18+ with Yarn package manager

## Installation

NOTE: Istall arcium cli for your system from the following page: https://docs.arcium.com/developers/installation 

```bash
# Clone the repository
git clone github.com/arnabnandikgp/matching-engine
cd matching_engine

# Install dependencies
yarn install

# Build Anchor program
arcium build

```

## Local Development

### Start Arcium Localnet
```bash
# Start local Arcium MPC network (in separate terminal)
arcium localnet
```


## Testing

Run the comprehensive test suite:

```bash
# Run all tests
anchor test

# Run specific test file
yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/matching_engine.ts

# Run with verbose logging
anchor test -- --grep "pattern"
```

Test suite covers:
- Core functionality (initialization, vaults, orders, matching, settlement)
- Edge cases (validation, boundaries, error handling)
- Performance (load testing, throughput)
- Security (privacy verification, access control)
- Integration (end-to-end user journeys)

See [TESTING_STRATEGY.md](./TESTING_STRATEGY.md) for detailed testing documentation.

## Project Structure

```
matching_engine/
├── encrypted-ixs/              # MPC computation circuits
│   └── src/
│       └── lib.rs              # submit_order, match_orders logic
├── programs/matching_engine/   # Solana on-chain program
│   └── src/
│       ├── lib.rs              # Program entrypoint & callbacks
│       ├── instructions/       # Instruction handlers
│       │   ├── initialize.rs
│       │   ├── submit_order.rs
│       │   ├── trigger_matching.rs
│       │   └── execute_settlement.rs
│       └── states/             # Account structures
│           ├── order_book_state.rs
│           ├── order_account.rs
│           └── vault_state.rs
├── tests/                      # Integration tests
├── Anchor.toml                 # Anchor configuration
└── Arcium.toml                 # Arcium network configuration
```

## Key Concepts

### Encryption Types
- `Enc<Shared, T>` - Encrypted data shared between user and MPC network
- `Enc<Mxe, T>` - Encrypted data only MPC network can decrypt

### Order Lifecycle
1. **Pending (0)** - Order account created, funds locked
2. **Processing (1)** - Added to encrypted orderbook
3. **Rejected (2)** - Orderbook full or validation failed
4. **Partially Filled (3)** - Matched but not fully filled
5. **Fully Filled (4)** - Completely matched and settled

### Nonce Management
Every MPC operation requires a nonce and produces a new nonce. The program tracks:
- `orderbook_nonce` - Current nonce for orderbook encryption
- `match_nonce` - Fresh nonce for each match result

Critical: Callbacks must update stored nonces or subsequent operations will fail.

### User Pubkey Passing
Arcium has no native pubkey type, so public keys are passed as 4x `u64` chunks:
```rust
// Split pubkey into chunks
let chunks = [
    u64::from_le_bytes(pubkey[0..8]),
    u64::from_le_bytes(pubkey[8..16]),
    u64::from_le_bytes(pubkey[16..24]),
    u64::from_le_bytes(pubkey[24..32]),
];

// Reconstruct in MPC
let pubkey = reconstruct_from_chunks(chunk0, chunk1, chunk2, chunk3);
```

## Configuration

### Orderbook Limits
- `MAX_ORDERS = 10` (per side)
- `MAX_MATCHES_PER_BATCH = 5`
- Matching rate limit: 15 seconds between triggers

### Account PDAs
- OrderBookState: `[b"order_book_state"]`
- OrderAccount: `[b"order", order_id, user_pubkey]`
- VaultState: `[b"vault", mint, user_pubkey]`

## Documentation

- [ARCHITECTURE_DIAGRAM.md](./ARCHITECTURE_DIAGRAM.md) - System architecture overview
- [COMPLETE_FLOW_DIAGRAMS.md](./COMPLETE_FLOW_DIAGRAMS.md) - Detailed flow diagrams
- [TESTING_STRATEGY.md](./TESTING_STRATEGY.md) - Test suite documentation
- [COMPREHENSIVE_TEST_CHECKLIST.md](./COMPREHENSIVE_TEST_CHECKLIST.md) - Full test checklist

## Security Considerations

**Privacy Guarantees:**
- Order amounts and prices never stored in plaintext on-chain
- Orderbook structure hidden in encrypted ciphertext
- Only matched parties learn execution details

**Known Public Information:**
- Order IDs (for tracking)
- User public keys (for vault derivation)
- Order types (buy/sell - not considered alpha)
- Timestamps and statuses (for UX)

**Trust Assumptions:**
- Arcium MPC network operates honestly
- Backend settlement authority executes settlements correctly
- Users protect their encryption keys

## License

GPL v3

## Contributing

Contributions are welcome. Please see [COMPREHENSIVE_TEST_CHECKLIST.md](./COMPREHENSIVE_TEST_CHECKLIST.md) for testing requirements before submitting PRs.

## Acknowledgments

Built with [Arcium](https://arcium.com) - Confidential Computing Network for Blockchain
