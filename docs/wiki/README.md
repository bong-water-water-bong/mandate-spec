# Project Wiki: mandate-spec

> **Agent onboarding:** Read this file first. It tells you what the project is, how to run it, and what's in flight. You should not need to guess.

## Mission
Open specification for **Mandate v0** — a signed, attested purchasing mandate for the buyer side of agentic commerce. A mandate travels from a buyer's agent into any merchant without requiring a prior relationship, carrying wallet identity, structured intent, budget cap, settlement-rail preferences, and an SEV-SNP attestation of the agent runtime that minted it.

This fills a gap in merchant-side agentic-commerce stacks (Stripe Link Wallet, Visa Trusted Agent, Mastercard Agent Pay, Google ACP, OpenAI Instant Checkout): they define how merchants accept agent traffic, but not how a buyer's agent travels safely into any merchant.

## Architecture
```
mandate-spec/
├── crates/
│   ├── mandate-core/     ← Rust: Mandate struct, signing, verification
│   ├── mandate-attest/   ← SEV-SNP attestation of the agent runtime
│   └── mandate-mint/     ← CLI: mint a mandate from a wallet
├── packages/
│   └── mandate-js/       ← TypeScript SDK: create, verify, parse mandates
├── docs/specs/           ← canonical design docs
│   └── 2026-05-03-mandate-v0.md
└── examples/             ← integration examples
```

## Key Concepts
- **Wallet-signed**: mandate is signed by the buyer's wallet key — merchant can verify without trusting the agent platform
- **Budget-capped**: `max_amount` + `currency` + `expiry` fields limit what the agent can spend
- **Rail-agnostic**: `settlement_rails[]` lists accepted payment networks (ACH, card, crypto, etc.)
- **SEV-SNP attested**: the `attestation` field proves the agent runtime is unmodified and trustworthy
- **Content-addressed**: mandate hash = SHA-256 of canonical JSON payload; immutable once minted

## How to Build / Test
```bash
# Build Rust crates
cargo build --all

# Run tests
cargo test --all

# Mint a demo mandate
cargo run -p mandate-mint -- --wallet demo --amount 50.00 --currency USD

# TypeScript SDK
cd packages/mandate-js && npm install && npm test
```

## Current State
- v0 draft spec in `docs/specs/2026-05-03-mandate-v0.md`
- Rust crate scaffold in `crates/`
- TypeScript SDK in `packages/mandate-js/`
- Not production-ready — spec is in active design

## Invariants
- **Mandates are immutable**: once minted and signed, no field can change. Create a new mandate for a new transaction.
- **Budget cap is hard**: agents must not exceed `max_amount`. Any implementation that ignores the cap is non-compliant.
- **Attestation is required in v1**: in v0 attestation is optional; in v1 it will be required.

## Related
- `docs/specs/2026-05-03-mandate-v0.md` — full v0 design
- [[lemonade-cashier]] — intended first merchant-side integration target
