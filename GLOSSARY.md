# Glossary — self-custody-wallet

Domain language for this project. Implementation details — table names,
column names, file paths, RPC method names — live in the code; this file
holds the terms and what they mean.

## Wallet

The HD wallet record owned by `walletd`: a database row holding derivation
metadata, per-chain key labels, and audit linkage. No private material lives
here; the seed lives only in `signerd`. This is what a user creates or imports
in the UI. One Wallet maps to one SmartAccount per chain (same HD seed,
different derivation path), one-to-many across chains.

## SmartAccount

The deployed `WalletAccount` Solidity contract at a specific address on a
specific chain. It has an owner (a derived HD key), a validator module set
(including SessionKeyValidator), and a GuardianModule. A SmartAccount is an
ERC-4337 account; the two terms refer to the same entity — SmartAccount is
the domain term, ERC-4337 account is the protocol term.

## Transaction

A pending or completed operation that moves value or invokes a contract
through the wallet. Three kinds share one lifecycle —
DRAFT → THREAT_CHECKED → AWAITING_APPROVAL → SUBMITTED → CONFIRMED | FAILED | REVERTED:
a direct EVM transfer signed by a derived key, an ERC-4337 UserOperation
submitted through a bundler, and a Solana native or PDA-gated transaction.

## Intent Mandate

A typed structure the agent emits from natural language, shaped after AP2
(Google, Sept 2025 — an authorization-layer protocol for agentic payments built
on signed Mandates proving user intent). Four fields: who (recipient), what
(action), limit (spend cap in wei or token base unit — never human-readable
decimals), expiry (when this intent lapses). Never a raw payload; schema
violations are rejected without a repair loop.

The `what` field is a typed discriminated union:

| `action` | Meaning |
|---|---|
| `transfer_eth` | Send ETH to `who`, amount bounded by `limit` |
| `transfer_erc20` | Send an ERC-20 token to `who`, amount bounded by `limit` |
| `approve_erc20` | Grant an ERC-20 allowance to `who`, amount bounded by `limit` |
| `contract_call` | Call a contract at `who` with encoded `data`; `limit` is ETH value |

`swap` is intentionally absent — the agent decomposes it into a
`contract_call` with the router as `who`.

See ADR 0001 for why the Mandate is unsigned.

This repo uses the AP2 *shape*, not the full AP2 protocol: the Intent Mandate is
the **intention** layer and is not itself signed. The **authorization** layer is
the on-chain signature — the session key signs the UserOp hash, or `signerd`
signs an EOA transaction. Keeping intention and authorization as separate layers
is the point of the parent plan's thesis ("authorization is not intention"): a
hijacked agent can produce a validly-signed UserOp that passes policy, so the
defense is the deterministic policy engine plus on-chain session-key limits, not
a second signature on the intent.

## Guardian

An Ethereum address registered in `GuardianModule` of a SmartAccount, empowered
to co-sign recovery actions under an M-of-N threshold. On-chain, a guardian is
only an address; `walletd` holds a human-readable label for each address for the
UI and audit log.

## Session Key

An ephemeral signing key pair the agent generates and registers on-chain with
SessionKeyValidator. The private half stays with the agent; the public address
and its permission set (allowed targets and selectors, per-transaction spend
cap, expiry) form the on-chain credential. The two facets are one credential:
the agent's key pair is the session key, and the on-chain record is what makes
that key a *session* key rather than an arbitrary key.
