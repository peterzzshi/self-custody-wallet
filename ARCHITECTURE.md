# Architecture — `self-custody-wallet`

This document carries the gap-closing narrative and the architectural decisions
with their alternatives. It is the durable record that remains after
`IMPLEMENTATION_PLAN.md` is deleted at M10.

> **Status**: stub. Decisions settled during design are recorded in
> `docs/adr/` and `GLOSSARY.md`; this document synthesizes them at completion.
> The sections below name what each must contain so the M10 pass does not
> start from a blank page.

## The two-process split

`walletd` (Go) parses untrusted input and owns all state; it never holds the
seed. `signerd` (Rust) holds the seed and nothing else: no listening socket,
supervised by `walletd` over stdio, derives per request, signs, zeroizes.
A crash in `walletd` cannot leak a key it never had.

**Honest residual risk**: root inside the `signerd` container can read the
seed; stdio IPC trusts the parent; no hardware root of trust. The production
mapping (PKCS#11 / cloud KMS / secure enclave; MPC-TSS) is documented, not
simulated.

## Key custody comparison

The table `ARCHITECTURE.md` must carry at M10 — columns: where the key lives at
rest, where it materialises at signing, what an attacker must steal, deployment
complexity. Rows: HD (single key, process-isolated), custodial (bank model),
hardware wallet (TEE boundary), MPC-TSS (key never exists, GG20 ceremony),
self-custody multisig (key exists per signer), ERC-4337 account abstraction.

## MPC and the signing boundary

This repo does not implement MPC. `crypto-chain-lab` demos Shamir (key
reconstructs); production MPC (GG20/CGGMP21) keeps the key from ever existing.
The honest residual-risk paragraph names what MPC would change: `signerd`
becomes one party in a t-of-n protocol; the seed is replaced by a share
generated without a dealer; signing becomes a multi-round network protocol.

## Agent trust model

The LLM proposes; a deterministic policy engine disposes; the signing key never
enters the model's context. The Intent Mandate is unsigned intention
(see ADR 0001); authorization is the on-chain signature alone. The defense
against a hijacked agent is the deterministic policy engine plus the on-chain
session-key cap, not a second signature on the intent.

## Decision log

- `docs/adr/0001-unsigned-intent-mandate.md` — why the Intent Mandate is
  unsigned intention, not a signed AP2 credential.

Open decisions to resolve before the listed milestone, with defaults:

1. EIP-7702 delegated EOAs — out of scope (M4).
2. ZK attestation — deferred; owned by `rwa-tokenization-platform` and
   `crypto-chain-lab` (M10).
3. Multi-instance `walletd` — single instance, multi-instance correctness
   argument stated honestly (M3).

## Boundary with `crypto-payment-service`

That project owns custody of money: balances, deposit crediting, a
forward-only transaction state machine, custody-provider adapters. This repo
owns the layer those providers abstract over: authorizing actions, holding
keys, deriving addresses, allocating nonces, broadcasting. It holds no ledger.
