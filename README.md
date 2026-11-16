# self-custody-wallet

A self-custody wallet that closes three hard-knowledge gaps from production
custody work: the signing layer, ERC-4337 account abstraction from spec, and
an AI pipeline where a wrong output is irreversible.

## Status

**Rewrite in progress.** The previous implementation in this repository was a
GMGN clone (TypeScript frontend/backend). It has been reset. The current work
follows the phased plan in `IMPLEMENTATION_PLAN.md` — see there for milestones,
scope, and the gap-closing narrative.

This is a portfolio project, not production software. It is unaudited and
non-production; the available evidence is invariants, test vectors, and bundler
conformance, not deployment.

## What it is

- **`signerd`** (Rust) — process-isolated key custody: BIP-32/39 HD derivation,
  sealed seed, zeroize after use, no network surface.
- **`walletd`** (Go) — orchestration: HTTP API, Postgres state, nonce leases,
  outbox, threat analysis, policy engine, audit log.
- **`contracts/evm`** (Foundry) — ERC-4337 `WalletAccount` with modular
  validators: `SessionKeyValidator`, `GuardianModule`, `MockPaymaster`.
- **`contracts/solana`** (Anchor) — PDA smart account (pre-declared cuttable).
- **`services/agent`** (Python) — intent → policy → session key → on-chain
  execute, with adversarial evals and zero-policy-violation as the finish line.

See `ARCHITECTURE.md` for the design decisions and `IMPLEMENTATION_PLAN.md`
for the milestone breakdown.

## Quickstart

```sh
make up       # docker compose up: Postgres + Anvil + Alto bundler
make test      # unit tier, hermetic
make test-integration  # tagged integration tier
```

## License

MIT — see `LICENSE`.
