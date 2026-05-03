# mandate-spec

Open spec for **Mandate v0** — a signed, attested purchasing mandate that travels from a buyer's agent into any merchant.

A mandate is the **buyer-side primitive** that complements merchant-side agentic-commerce stacks (Stripe Link Wallet, Visa Trusted Agent, Mastercard Agent Pay, Google ACP, OpenAI Instant Checkout). It carries the buyer's wallet identity, structured intent, budget cap, settlement-rail preferences, and an SEV-SNP attestation of the agent runtime that minted it — so any merchant can verify and quote it without a prior relationship to the buyer or their agent.

## Why this exists

Every merchant-side agentic-commerce stack today is asking the same question: *how do we accept agent traffic safely?* None of them is shipping the inverse: *how does the buyer travel safely into any merchant?*

A mandate is that artifact. It's content-addressed, wallet-signed, runtime-attested, budget-capped, and rail-agnostic. Merchants verify it without trusting the agent platform; buyers retain custody of their wallet and their preferences.

## Status

**v0 — draft.** See [`docs/specs/2026-05-03-mandate-v0.md`](docs/specs/2026-05-03-mandate-v0.md) for the full design.

## Layout

```
docs/specs/                      design docs (canonical)
schema/                          JSON Schema for the mandate body
examples/                        worked example mandates
crates/mandate-core/             Rust types + sign/verify + attestation (stub)
packages/mandate-core-ts/        TypeScript types + sign/verify (stub)
```

## Settlement

The spec is rail-agnostic. The reference settlement implementation lives in [`1bit-pay`](https://github.com/bong-water-water-bong/1bit-pay), which adds a `mandate_settle` ledger entry kind and a mandate-bound `pay_receipt` shape that closes the audit loop back into [`1bit.vault`](https://github.com/bong-water-water-bong/1bit.vault).

## License

MIT — see [`LICENSE`](LICENSE).

## Contributing

This is a v0 draft. Issues and proposals welcome on the spec, the schema, the canonical-serialization rules, and the attestation profile. Implementation PRs against the Rust and TypeScript crates are welcome once v0.0.1 milestones are scoped.
