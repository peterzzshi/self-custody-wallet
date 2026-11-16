# Implementation Plan — `self-custody-wallet`

Phased build plan. Delete when no phased work remains (`WORKFLOW.md`), after
moving durable decisions into `ARCHITECTURE.md`.

**Last revised**: 2026-11-16, after gap-driven reframe. See §Divergence From The
Parent Plan — the parent documents are now *industry-gap reference only*, not a
technical spec.

## What This Project Is

This repo closes three hard-knowledge gaps that the rest of the portfolio and the
Chaos Theory production experience do not cover:

1. **The signing layer.** At Chaos Theory, Fireblocks owned key derivation, nonce
   allocation, signing, and rebroadcast. The Go service called `sendcoins`. This
   repo builds exactly that layer: BIP-32/39 HD derivation in Rust, an isolated
   signing process, nonce leases against a live node, and a broadcast pipeline that
   survives `kill -9`.

2. **ERC-4337 from spec.** No repo in the portfolio has a line of Solidity. No repo
   has read `validateUserOp` or ERC-7562 from the spec. M4 is the first hand-written
   smart contract work: an account that passes a production bundler's simulation and
   whose session-key validator enforces spend limits on-chain.

3. **An AI pipeline where a wrong output is irreversible.** M8 builds the safety
   architecture: typed intent schema, adversarial eval corpus, deterministic policy
   gate, zero policy-violation rate. The differentiator is not the LLM surface; it is
   applying the same judgment used in production payments — gateway enforces, service
   trusts nothing — to an AI surface.

One sentence to defend: *"I owned custody integration in production, Fireblocks did
the signing layer; here I built that layer, then put an ERC-4337 account and a
safety-first agent on top of it."*

## Gaps This Closes

| Gap | Where it shows up in interviews | What closes it |
|---|---|---|
| HD derivation and the signing pipeline | "Walk me through how a key becomes a signed tx" | M1: BIP-39 → BIP-32 → ECDSA/ed25519 in Rust, stdio boundary, zeroize |
| Nonce ownership against a live node | "You mentioned Fireblocks allocated nonces — can you do that yourself?" | M3: `SELECT … FOR UPDATE SKIP LOCKED`, idempotency record, outbox, reconciler |
| Solidity / EVM smart contract writing | Zero contracts anywhere in portfolio | M4: WalletAccount, SessionKeyValidator, GuardianModule from spec |
| ERC-4337 validation discipline (ERC-7562) | "Why can't validateUserOp read from storage?" | M4: storage-access test, bundler simulation end-to-end |
| AI pipeline with irreversible outputs | "How do you stop the agent from over-spending?" | M8: policy engine, session key on-chain limit, adversarial corpus, zero-violation metric |
| MPC threat model (conceptual) | "What would it take to make signerd production-grade?" | ARCHITECTURE.md: Shamir vs TSS, what MPC changes, why the gap stays open honestly |

**What is not a gap and therefore not a centerpiece**: Go orchestration, Postgres
state machines, event-driven pipelines, custody-provider integration. Those are
already proven by production experience and other repos. They appear here because
they are the correct foundation, not because they are the story.

## Scope

| In scope | Out of scope (and why) |
|---|---|
| Go orchestrator + Rust signer (two processes) | Three-process splits of any kind — cap at two |
| EVM ERC-4337 account written from spec, with modular validators | Importing a stock audited account implementation as the deliverable |
| Session keys enforced on-chain; guardian recovery | MPC / threshold signing — see §MPC and the Signing Boundary |
| Pre-signature threat analysis + fixture corpus + FP measurement | Own EntryPoint, bundler, or paymaster — consume the canonical ones |
| One Anchor PDA smart account on Solana | Bitcoin adapter — argued in `ARCHITECTURE.md`, not built |
| Agent scoped to the session-key seam | Chatbot surface, RAG, fine-tuning |
| `docker compose up` on Anvil + real local bundler | L1↔L2 bridge, testnet/mainnet deployment, any scale-out |

### Bitcoin: prose only

`bitcoin-indexer` already owns UTXO/Taproot/reorg. A wallet adapter would compete
with our own repo. The unique residue — PSBT construction, sighash signing — is
roughly a week for a paragraph of signal. Instead, `ARCHITECTURE.md` carries the
argument: script policy is immutable at address creation; program policy is
upgradeable, which is more capable for the user and adds the upgrade authority as
a new attack surface. The `Signer` seam is designed so a Bitcoin adapter is
cheap to add later; that design decision is what is being bought.

### Boundary with `crypto-payment-service` (workspace rule)

That project owns *custody of money*: balances, deposit crediting, a forward-only
transaction state machine, withdrawal approvals, custody-provider adapters. It
demonstrates integration against Fireblocks and BitGo. **This repo owns the layer
those providers were abstracting over** — authorizing actions, holding keys,
deriving addresses, allocating nonces, broadcasting — and holds no ledger. No
user balances, no crediting, no reconciliation of funds. Stated in
`ARCHITECTURE.md` so the two read as complementary, not competing.

### MPC and the signing boundary

`crypto-chain-lab` implements Shamir Secret Sharing — a dealer splits the key,
parties reconstruct at x=0, the **full private key materialises in memory** at
signing time. That is key splitting, not MPC/TSS. Production MPC (Fireblocks,
ZenGo) is structurally different: DKG generates shares without a dealer so the
key **never exists**, and collaborative signing (GG20/CGGMP21 for secp256k1)
combines partial signatures without ever reconstructing the key. The on-chain
signature is a normal ECDSA signature; there is no indication an MPC wallet was
involved.

This repo does **not** implement MPC. Reasons, stated plainly:

- A production GG20/CGGMP21 implementation requires Paillier encryption, oblivious
  transfer, range proofs, and a multi-round network protocol. The reference
  `tss-lib` is ~20k lines and took a team years. Building it solo for a portfolio
  is not credible, and an interviewer who knows the space asks "was it audited?"
  — the honest answer is no.
- Implementing Shamir and labelling it MPC is the common portfolio mistake. It
  reads worse than silence to anyone who knows the distinction, because the
  distinction is exactly what a custody interviewer probes.

What this repo does instead:

- `signerd` implements the **process-isolated single-key** boundary honestly:
  sealed seed, unseal per request, `zeroize`, no network surface. This closes
  the local attack surface for the demo and is the correct foundation.
- `ARCHITECTURE.md` carries the **key-custody comparison table**: HD
  (single key, process-isolated), custodial (bank model), hardware wallet (TEE
  boundary), MPC-TSS (key never exists, GG20 ceremony), self-custody multisig
  (key exists per signer). Columns: where the key lives at rest, where it
  materialises at signing, what an attacker must steal, deployment complexity.
  This is what an interviewer actually probes — not whether you can implement
  GG20, but whether you understand the threat model of each design.
- The **honest residual risk** paragraph names what MPC would change: in
  production, `signerd` becomes one party in a t-of-n protocol; the seed is
  replaced by a share generated without a dealer; signing becomes a multi-round
  protocol with network communication between parties; GG20/CGGMP21 are the
  production candidates for secp256k1.

Narrative thread, defensible under probing: *"At Chaos Theory I consumed
Fireblocks MPC — the key never materialised, Fireblocks ran the ceremony.
`crypto-chain-lab` demos Shamir — the key reconstructs, which is the thing MPC
specifically avoids. `signerd` is an honest step toward that boundary:
process-isolated, no network surface, zeroize after use. `ARCHITECTURE.md`
documents what a full MPC upgrade changes and why this repo stops short of it."*

## Architecture

```
                    HTTP (signed requests, idempotency keys)
  UI (React/TS) ──────────────────────────────►  walletd (Go)
                     OpenAPI-generated client      │
                                                   ├── Postgres: accounts, tx state,
                                                   │    nonce leases, outbox, audit
                                                   ├── JSON-RPC: Anvil / RPC endpoints
                                                   └── stdio pipe (length-prefixed)
                                                          │
                                                          ▼
                                                    signerd (Rust)
                                                    - holds sealed seed
                                                    - derives on demand
                                                    - signs, zeroizes
                                                    - NO listening socket
                                                    - NO DB, NO network

  contracts/evm      (Foundry)  WalletAccount + validators + recovery
  contracts/solana   (Anchor)   PDA smart account
  services/agent     (Python)   intent → policy → session key → on-chain execute
  bundler + paymaster (Alto/Rundler + a mock paymaster) in docker-compose
```

**Why two processes.** The isolation claim has to be structural or it is
marketing. `walletd` parses untrusted input, so it must never hold the seed.
`signerd` holds the seed and nothing else: no listening socket, supervised by
`walletd` over stdio, derives per request, signs, zeroes the buffer, returns
signature bytes. Its dependency tree stays deliberately tiny so the auditable
surface is small. A crash in `walletd` cannot leak a key it never had.

**Honest residual risk**, stated in `ARCHITECTURE.md`: root inside the `signerd`
container can read the seed; stdio IPC trusts the parent; no hardware root of
trust. The production mapping (PKCS#11 / cloud KMS / secure enclave, where the
key is unexportable and the boundary is hardware-enforced; MPC-TSS, where the
key never exists) is documented, not simulated. Naming the limit reads as more
senior than claiming an HSM.

## Layout

```
self-custody-wallet/
  services/
    walletd/                 # Go 1.23 — API, adapters, tx pipeline, policy, audit
      cmd/walletd/
      internal/
        api/                 # HTTP handlers, request signing, idempotency
        account/             # account model, per-chain metadata
        adapters/evm/        # UserOp construction, viem-equivalent encoding
        adapters/solana/     # PDA derivation, instruction building
        tx/                  # state machine, nonce leases, outbox, reconciler
        threat/              # deterministic analyzers + fixture corpus
        policy/              # spend caps, allowlists, approval thresholds
        signer/              # signerd client (stdio protocol)
        audit/               # append-only event log
        store/               # pgx queries, migrations
    signerd/                 # Rust — key custody, derivation, signing
      src/{main,rpc,keystore,derive,zeroize_guard}.rs
    agent/                   # Python 3.12 + FastAPI — intent, policy, evals
  contracts/
    evm/                     # Foundry: WalletAccount, SessionKeyValidator,
                             # GuardianModule, MockPaymaster
    solana/                  # Anchor: smart account program
  packages/shared/           # generated TS client from openapi.yaml
  deploy/                    # docker-compose, anvil state, entrypoint pinning
  README.md  ARCHITECTURE.md  IMPLEMENTATION_PLAN.md
```

## Stack, Fitted To Purpose

| Concern | Choice | Fit rationale (and what was rejected) |
|---|---|---|
| Orchestration, HTTP, state machines | **Go 1.23** | Goroutines + `pgx` + explicit SQL suit a signing queue and nonce leases; the dominant language of real custody backends. Not new to the portfolio — chosen because it is correct, not because it is novel |
| Key custody + signing | **Rust** | Justified by the *role*, not novelty: no GC pauses, `zeroize` for memory hygiene, tiny dependency tree, no network stack. New as a *long-lived service* shape — `crypto-chain-lab` and the AMM use Rust, but neither runs a supervised process with a sealed secret |
| EVM encoding | **`go-ethereum`** types + hand-rolled UserOp v0.7 packing | Writing the packing is the interview surface. `abi` for selectors and calldata. Rejected `viem` because it lives in TS |
| Contracts | Solidity 0.8.28, **Foundry** | Unit + invariant speed. **New capability** — no Solidity exists anywhere in the portfolio; this is the gap closing |
| Solana | **Anchor**, `litesvm` | PDA arithmetic and `invoke_signed` are the point. `concentrated-liquidity-amm` already uses Anchor; the PDA smart account is a different program, not a repeat |
| IPC | Length-prefixed JSON over stdio, child supervised by `walletd` | No listening socket at all. Rejected unix socket (still a socket, needs FS-permission reasoning) |
| Storage | Postgres 16 + `pgx`; `golang-migrate` | One store for state, leases, outbox, audit. Rejected an ORM — explicit SQL is clearer for leases and `FOR UPDATE SKIP LOCKED` |
| Agent | Python 3.12, `uv`, FastAPI, Pydantic v2, LiteLLM | **New capability** — no Python in the portfolio. The one place Python is right: eval tooling and model APIs |
| UI | React + Vite + **wagmi** | `wagmi` for connect/chain switching only; all signing goes through `walletd` |
| Local chain | **Anvil** + **Alto** bundler + mock paymaster, all pinned | A demo depending on a faucet fails live. Deterministic boot in one command |
| Tests | Go `testing` + `testcontainers`, `forge test`, `pytest` | Hermetic unit tier; tagged integration tier |

## Invariants No Milestone May Violate

1. **The seed never leaves `signerd`.** No seed, private key, or derived secret
   in any HTTP response, log line, span attribute, error message, or DB row.
   A redaction test asserts this against a known test mnemonic.
2. **`walletd` never holds a key.** It sends `(key_id, path, payload)` and
   receives signatures. If it needs a key, the design is wrong.
3. **The agent cannot transition state.** It proposes; `walletd` owns every
   transition: `DRAFT → THREAT_CHECKED → AWAITING_APPROVAL → SUBMITTED →
   CONFIRMED | FAILED | REVERTED`.
4. **Policy is deterministic and out of the model's reach**, mirrored on-chain for
   session keys so a bypassed engine still cannot over-spend.
5. **No new nonce is consumed without an idempotency record.** Exactly-once
   broadcast at the *transaction* level; at-least-once is not acceptable for
   signing.
6. **Fail closed.** Signer down, EntryPoint unverified, migration pending, clock
   skew beyond replay window → refuse, do not degrade to signing.
7. **Deterministic before probabilistic.** Threat findings augment and can block;
   they never alone authorize.

---

## Milestones

### M0 — Reset and baseline

- Commit the working-tree deletions as one commit ("Reset implementation for
  rewrite"). Keep history — provenance is statable in an interview and the plan
  requires origin honesty. README states the rewrite; no disguising it.
- Add `.gitignore` (Go, Rust, Python, Foundry, Anchor, Node, env, IDE),
  `LICENSE`, `README.md` and `ARCHITECTURE.md` stubs.
- `go.work` / `cargo` workspace / Foundry / Anchor / `pyproject` skeletons;
  `openapi.yaml` with one health endpoint; `docker-compose.yml` with Postgres and
  the pinned bundler.
- `Makefile` with `make up`, `make test`, `make test-integration`.

**Accept**: clean clone → `make up` boots Postgres + bundler + Anvil with a
pinned EntryPoint; `make test` runs an empty suite green; no secrets in tree.

### M1 — `signerd`: the key boundary

**This is the first gap closer. Do not rush it.**

- Stdio protocol: versioned frames, request id, `Derive`, `SignDigest`,
  `SignUserOpHash`, `SignTransaction`, `Health`. Monotonic request ids; reject
  replays.
- Keystore: seed sealed at rest by a passphrase-derived key (Argon2id →
  AES-256-GCM, versioned envelope). Unseal per request, `zeroize` after.
  Unseal rate limited; failed attempts backed off and audited.
- Derivation in Rust: BIP-39 seed → BIP-32 (`m/44'/60'`, `m/84'/0'`) and
  SLIP-0010 ed25519 (`m/44'/501'`). Curve ops from audited crates
  (`k256`, `ed25519-dalek`, `bip32`, `tiny-bip39`, `zeroize`).
- `secp256k1`: ECDSA over a 32-byte digest (low-S normalized) and
  EIP-191/EIP-712 hashing. Solana: ed25519 over message bytes.
- No network code, no allocator-heavy surface, deny-by-default dependency
  review recorded in the README.

**Accept**: BIP-39/BIP-32/SLIP-0010 published test vectors as fixtures;
known-address vectors for EVM (EIP-55), Solana, and Bitcoin P2WPKH/P2TR
derivation (derivation only, no adapter); wrong passphrase fails closed; a test
greps the `signerd` log stream and binary output for the test mnemonic's words
and derived hex and finds nothing.
**Defend**: what the stdio boundary buys over an in-process interface; what a
real HSM changes; why the seed is sealed rather than merely held in memory;
what MPC would change here and why this repo stops short of it.

### M2 — `walletd` skeleton, store, and audit

- `openapi.yaml` as the contract; generated TS client lands in `packages/shared`.
- Schema: `accounts`, `derived_keys` (public half + label only), `transactions`,
  `tx_state_transitions`, `nonce_leases`, `outbox`, `audit_events`. Migrations
  forward-only.
- HTTP: create/import account, list accounts and balances, propose transaction,
  sign, broadcast, get status. Every mutation writes an audit event with a
  correlation id.
- Request auth: signed requests (HMAC over body + timestamp + nonce), replay
  window enforced, per-key rate limit.
- Startup validation: pinned EntryPoint address asserted `eth_getCode` non-empty;
  clock skew check; signer health check.

**Accept**: the same mnemonic yields per-chain account lists; audit row per state
change; replay of a captured request is rejected; no key bytes in any response
(asserted by fixture test).
**Defend**: idempotency-key semantics and why they differ from a transaction id;
why audit is append-only.

### M3 — Transaction pipeline: nonce leases, idempotency, outbox, reconciler

This closes the Fireblocks gap. `crypto-payment-service` demonstrates the state
machine and approval audit; this milestone demonstrates the layer Fireblocks was
hiding: nonce allocation against a live node, idempotent broadcast, and recovery
from the crash window between DB commit and RPC call.

- `nonce_leases`: `(chain_id, address, nonce)` claimed under
  `SELECT … FOR UPDATE SKIP LOCKED`, TTL, released on confirmed or reverted.
- Idempotency: client key → canonical request hash → stored outcome. Identical
  retry returns the same tx hash; the same key with a *different* body is a
  conflict, not a retry.
- Transactional outbox: state transition and broadcast intent committed together;
  a dispatcher drains it. No lost broadcasts on crash.
- Reconciler: periodic loop that resolves `SUBMITTED` transactions by receipt,
  reclaims orphaned leases, detects a tx replaced by a higher fee (same nonce)
  and records the replacement rather than corrupting state.
- Throttling and backpressure on RPC: bounded worker pool, circuit breaker on
  error rate, queue depth metric.

**Accept**:
1. 50 concurrent identical sends → exactly 1 transaction, 49 return the same hash.
2. 50 concurrent distinct sends → 50 transactions, 0 nonce collisions, all mined.
3. `kill -9` between DB commit and RPC broadcast, then restart → reconciler
   settles the transaction exactly once; no duplicate and no orphan.
4. RPC unavailable → requests queue or fail closed; no silent drop.
**Defend**: why a lease rather than optimistic locking; what breaks if two
`walletd` instances run (state it honestly); why the outbox pattern rather than
a two-phase commit; how this compares to the BitGo adapter in
`crypto-payment-service`.

### M4 — ERC-4337 account from spec, with modular validators

**This is the second gap closer.** First Solidity in the portfolio.

- `WalletAccount`: 4337 v0.7 `validateUserOp` over the packed UserOp hash,
  `execute`, `executeBatch`, upgradeability via UUPS.
- `SessionKeyValidator`: key + allowed `(target, selector)` set + per-token
  spend limit per period + expiry, validated against `userOp.callData`. **This is
  the on-chain limit a hijacked agent cannot exceed.**
- `GuardianModule`: M-of-N recovery with a delay, owner-gated guardian changes,
  key rotation through the recovery path.
- ERC-7562 discipline: validation phase performs no forbidden storage access and
  no external calls that could reenter. A test asserts the reachable storage set.
- Consume the **canonical EntryPoint** — do not deploy one.
- Foundry: unit tests + invariants (funds cannot move without a valid validation;
  a session key cannot exceed its cap; recovery cannot execute before its delay).

**Accept**: a UserOp built by `walletd` executes on local Anvil end-to-end
through the Alto bundler; over-cap session key reverts; recovery restores access
after the delay and cannot execute early; rotation preserves the account address;
bundler accepts the account with no `AA2x` validation failures.
**Defend**: why `validateUserOp` must not make trust assumptions about the
bundler; what ERC-7562 forbids and why; where 4337 stops being abstraction and
becomes a bundler/paymaster market; what an account upgrade can and cannot
protect.

### M5 — Threat gate before every signature

- Analyzers returning typed `Finding{code, severity, evidence, confidence}`:
  target contract age and bytecode size; proxy detection (EIP-1167, EIP-1967)
  and upgrade authority; unlimited `approve` / `setApprovalForAll`; `permit`
  with unbounded value; dangerous-selector set; brand-new recipient; current
  allowance state; gas-limit / priority-fee outliers; simulation revert reason
  and balance deltas.
- `eth_call` simulation before signing. A revert or a drain-shaped delta blocks
  the default path and requires an explicit audited override.
- Versioned fixture corpus as data files, not inline strings, so it grows:
  unlimited approve, approval+transferFrom drain, unverified fresh contract,
  `setApprovalForAll`, permit phishing, benign baseline set.
- Measured quality: precision/recall over the corpus, **false-positive rate
  reported**, and the user cost of a false positive argued in `ARCHITECTURE.md`.
- External `smart-contract-security-analyzer` integration over HTTP behind a
  flag; degrade to local analyzers when absent, and test the absent path.
- UI: findings panel with severity, evidence, plain-language explanation;
  override is explicit and audited.

**Accept**: every bad-calldata fixture is flagged with its expected code; the
benign set produces no high-severity finding; scanner-down still blocks the
drain fixture.
**Defend**: why deterministic checks first; the FP rate and what it costs a
user; why simulation alone is insufficient (a simulation can succeed and still
drain).

### M6 — Multi-approval, key rotation, recovery

- Threshold policy in `walletd`: above a value the transition is
  `AWAITING_APPROVAL`; a second approver signs the same canonical payload.
- On-chain enforcement for the guardian co-signing path (M-of-N over the UserOp
  hash) so the second approval is not merely a UI step.
- Rotation: rotate session keys, rotate owner via guardians, revoke a compromised
  session key immediately.
- Audited end-to-end trail: proposal → approval → execution with actor identity.

**Accept**: a high-value transfer cannot settle with one signature; the approval
verifies on-chain; rotation leaves balances and address intact; revocation takes
effect for the next UserOp.
**Defend**: claim strength should match the cost of duplication — why a UI gate
is insufficient and an on-chain gate is sometimes too rigid.

### M7 — UI

- React + Vite + wagmi: derive/import, per-chain accounts, balances, send flow
  with the threat panel gating "sign", approval screen, guardian management,
  session-key issuance with caps and expiry.
- Offline signing surface: sign locally, broadcast separately (covers the
  syllabus offline-generation topic).

**Accept**: the send flow cannot reach "sign" with an unresolved high-severity
finding; the whole flow is usable against local Anvil without a faucet.
**Defend**: what the UI is allowed to trust, and why analysis lives server-side.

### M8 — Agent scoped to the session-key seam

**This is the third gap closer.** First Python and AI pipeline in the portfolio.

The agent is a client holding a session key. The wallet enforces on-chain. No
chatbot, no RAG, no streaming UI. Steps A1–A3 build in parallel with M2–M3 and
need no wallet code — they produce a versioned eval artifact before the agent
loop exists.

| Step | Build | Acceptance |
|---|---|---|
| A1 | FastAPI skeleton, typed settings, Postgres, Alembic, Dockerfile, one traced structured-output call | Deterministic config; one call traced end-to-end |
| A2 | Typed intent parser — AP2-style Intent Mandate (who, what, limit, expiry); never a raw payload | Schema violations rejected without a repair loop |
| A3 | **Evals first**: ~200 golden intent→transaction pairs, deterministic scorer, one LLM-as-judge only where a rule cannot decide | Baseline accuracy recorded *before* the agent loop exists |
| A4 | Tool loop with hard step limits, repeated-call detection, token budgets, circuit breakers; tools: balance, gas, token metadata, quote | Loop cannot exceed budget; tool failure degrades, never invents a number |
| A5 | Deterministic policy engine + HITL state machine; adversarial corpus (poisoned token metadata, injected instructions in fetched content) | Policy-violation rate **0** on the adversarial set; model never transitions state |
| A6 | Wallet integration: agent holds a session key; policy mirrored on-chain; threat analysis consulted before proposing execution | An over-cap action fails on-chain even with the engine bypassed |

Recorded as measured numbers: intent accuracy, policy-violation rate (must be 0),
refusal correctness, p95 policy-decision latency, cost per intent.
**Defend**: workflow-vs-agent decision rule (A1–A3 is a workflow; the loop is
introduced only where the path is genuinely not enumerable upfront); hallucination
taxonomy applied to the parser; *authorization is not intention* — name the hijack
scenario and the deterministic gate that stops it; why evals precede the loop.

### M9 — Solana PDA smart account (pre-declared cuttable)

Ships only if M0–M8 are complete. Sole purpose: prove account abstraction is a
property, not an EVM feature. `concentrated-liquidity-amm` already uses Anchor for
a different program; this is not a repeat.

- Anchor: PDA `[b"account", authority]` holding config (spend limits, session
  keys, expiry), `execute` performing a CPI via `invoke_signed`,
  `add_session_key`, `recover`.
- `walletd` adapter: PDA derivation, instruction building; the ed25519 HD key is
  the *authority*, not the account.
- Tests on `litesvm`.

**Accept**: PDA executes a transfer gated by an on-chain limit; a session key
without the rule fails; recovery verified.
**Defend**: what a PDA cannot do that a 4337 account can; why `invoke_signed` is
the whole trick; no bundler/paymaster market on Solana.
**If cut**: `ARCHITECTURE.md` carries the documented argument instead.

### M10 — Documentation and completion gate

- `README.md`: problem, solution, features, stack, `make up`, testing commands,
  and the measured numbers.
- `ARCHITECTURE.md`: the gap-closing narrative + decisions with alternatives —
  two-process split, spec-written account, modular validators, policy placement,
  outbox + leases, threat-analysis design, consume-vs-own EntryPoint, Bitcoin
  comparison, MPC residual-risk paragraph, and the wallet-category comparison
  table (HD / MPC-TSS / custodial / hardware / multisig / AA — covering all
  syllabus Stage 4 categories).
- Delete this plan.
- Measured facts stay here. The seven architectural-design questions from
  `WORKFLOW.md` go to `../interview-deep-dives/self-custody-wallet.md`,
  labelled as extrapolation where they are.

## Dependency Order

M1 → M2 (nothing signs before the boundary is proved).
M4 before M5's override path.
M3 before M8-A6 (the agent must not race a broken pipeline).
M4 before M8-A6 (session keys are the seam).
M8-A1 through A3 are independent of M2–M4 and build in parallel.

**Cut order under pressure**: M9 → M6's on-chain co-signing (service-only with
gap documented) → M7 scope reduction. **Never cut** M1, M3, M4, M5, or the
agent's policy engine — those are the three gap closers.

## Measuring

Recorded in the repo as numbers, not adjectives:

- concurrency: duplicate-suppression and nonce-collision counts (M3)
- crash recovery: recovery outcome across N injected kill points
- bundler: UserOp success rate and validation errors on local Alto
- threat corpus: precision, recall, false-positive rate
- agent: intent accuracy, policy-violation rate, refusal correctness, p95
  latency, cost per intent
- signing: p50/p95 `signerd` latency, unseal rate

## Risks

| Risk | Mitigation |
|---|---|
| Two-process split burns days on IPC plumbing | Freeze the protocol in M1 as a versioned frame spec before implementation; do not evolve it mid-build |
| `signerd` balloons into a wallet | Hard rule: no DB, no network, no business logic. If a feature needs state, it belongs in `walletd` |
| 4337 from spec produces subtle validation bugs | Test against a real bundler from M4 day one, not at the end; invariant tests before feature tests |
| Rust crypto crate churn | Pin versions; depend only on `k256`, `ed25519-dalek`, `bip32`, `tiny-bip39`, `zeroize` |
| Agent absorbs unlimited effort | A1–A6 only; the golden set and zero-policy-violation metric are the finish line |
| Anvil/bundler version drift breaks the demo | Pin images by digest in compose; assert EntryPoint code at startup |
| "Built from spec" reads as "untested code" | Say it plainly: unaudited, non-production, with invariants and bundler conformance as the evidence available |
| Solana slip | Pre-declared cuttable; the cross-model claim reverts to documented analysis in `ARCHITECTURE.md` |

## Open Decisions (settle before the listed milestone)

1. **Per-period on-chain budgets** vs per-tx cap only — default: per-tx cap
   on-chain plus a rolling-budget mirror in `walletd`, gap documented. (M4)
2. **EIP-7702 delegated EOAs** — default: out of scope, reasoning in
   `ARCHITECTURE.md`. (M4)
3. **Approver identity model** — default: guardian quorum, reusing M4's module.
   (M6)
4. **ZK attestation** — default: deferred; `crypto-chain-lab` and
   `rwa-tokenization-platform` own the use case. (M10)
5. **Whether `walletd` supports more than one instance** — default: single
   instance, correctness argument for multi-instance stated honestly. (M3)

## Divergence From The Parent Plan

The parent documents are now **industry-gap reference**, not a technical spec.
Recorded here so later sessions do not re-litigate:

| Parent plan said | This plan does | Why |
|---|---|---|
| Bitcoin P2WSH/P2TR implemented in the wallet | Documented comparison only | Competes with `bitcoin-indexer`; low signal per build-week |
| L1↔L2 guardian recovery bridge | Out of scope | Demonstrates L2 mechanics, not the wallet's thesis |
| Wallet service in TypeScript | Split: Go orchestrator + Rust signer | Fit for purpose; the signing boundary is structural this way |
| ZK attestation integrated | Deferred | Owned by `rwa-tokenization-platform` and `crypto-chain-lab` |
| MPC wallet comparison implied by implementation | Comparison + residual-risk paragraph in `ARCHITECTURE.md` only | `crypto-chain-lab` does Shamir reconstruction, not MPC/TSS — the deferral was overstated; gap stays open and named honestly |
| Three-chain AA survey as the thesis | Two-chain (EVM deep, Solana scoped), thesis is the three gap closers | Depth per build survives an interview; breadth does not |
| Pre-transaction analysis as a feature list | Analyzers + fixture corpus + measured FP rate | The measurement is the contribution, not the checks |
| M8 (agent) is the last milestone | A1–A3 build in parallel with M2–M3 | Agent is a gap closer, not a garnish; it cannot end up cut by schedule |
