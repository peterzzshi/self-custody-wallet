# ADR 0001 — Intent Mandate is unsigned intention, not a signed AP2 credential

**Status**: accepted
**Date**: 2026-11-16

## Context

The agent's typed parser emits an Intent Mandate shaped after the AP2 protocol
(Google, Sept 2025), which specifies *signed* Mandates as cryptographic proof
of user intent. The plan borrows the four-field shape (`who`, `what`, `limit`,
`expiry`) but the question is whether the Mandate itself must be signed.

The core security thesis from the parent plan is "authorization is not
intention": a hijacked agent can produce a validly-signed UserOp that passes
every spending control, so the defense cannot be a second signature on the
intent — it must be a deterministic policy engine and on-chain session-key
limits that the model never touches.

## Decision

The Intent Mandate is **unsigned**. It is a typed data structure that records
what the LLM understood the user to want. It is validated by schema (schema
violations are rejected without a repair loop) and by the deterministic policy
engine. It is never itself a cryptographic authorization.

Authorization happens exactly once, at the layer where it is enforceable:
the on-chain signature. The agent signs the UserOp hash with its session-key
private key; `signerd` signs EOA transactions. Those signatures are the
authorization. The Intent Mandate is the intention layer upstream of them.

## Alternatives considered

**Sign the Intent Mandate with the user's key**: would give a user-side
cryptographic proof that this specific intent was authorized. Rejected because
(a) it requires the signing key to be accessible to the user at intent time,
coupling the UX to key management; (b) the signed Mandate would need to be
verified inside `walletd`, adding a second signature-verification path; and
(c) it still does not stop a hijacked agent from constructing a Mandate that
passes policy — the agent would just produce a fraudulent unsigned Mandate and
ask the user to sign it. The on-chain cap is the unforgeable limit.

**Use the full AP2 protocol**: AP2 includes transport, discovery, and
delegation mechanics beyond the Mandate shape. Out of scope: the wallet is not
an agentic payment network participant; it is a demo of the safety architecture.

## Consequences

- The M8-A2 Pydantic schema has four fields (`who`, `what`, `limit`, `expiry`)
  plus an `action` discriminant. No signature field.
- The policy engine receives the parsed Intent Mandate and returns a decision.
  The model never receives the policy decision as a prompt; it cannot observe
  or learn from it.
- The honest limitation: without a signed Mandate, there is no cryptographic
  proof that the user authorized this specific intent (as opposed to the
  broader session key). This is documented in `ARCHITECTURE.md` under "Agent
  trust model." The mitigation is HITL for high-value actions and the on-chain
  cap that cannot be exceeded regardless.
