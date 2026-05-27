# Attestations

Verifiers add credibility attestations to identity bonds. Only authorized attesters can add or revoke attestations; each attestation has a weight, timestamp, and is deduplicated and replay-protected.

## Data structure

- **Attestation** — `id`, `verifier` (attester address), `identity` (subject address), `timestamp`, `weight`, `attestation_data`, `revoked`. Stored by ID; dedup key is (verifier, identity, attestation_data).
- **Subject attestations** — Live, non-revoked attestation IDs for an identity. Revoked IDs are removed from this index.
- **Subject attestation count** — O(1) count per identity. This count is defined as the number of live, non-revoked IDs in `SubjectAttestations(identity)`.

## Authorization

- **register_verifier(verifier, stake_deposit)** — Stake-based registration (see verifiers.md).
- **deactivate_verifier(verifier)** / **deactivate_verifier_by_admin(admin, verifier)** — Disables attestation rights.
- **register_attester(attester)** / **unregister_attester(attester)** — Legacy admin-managed authorization (backwards compatible).
- **is_attester(attester)** — Returns whether the address is currently authorized to attest.

## Adding attestations

- **add_attestation(attester, subject, attestation_data, nonce)**  
  - Caller must be the attester (require_auth).  
  - Attester must be registered.  
  - `contract_id` and `deadline` are part of the signed action context and are validated before nonce consumption.  
  - Nonce must match current attester nonce (replay prevention); nonce is incremented on success.  
  - Signed actions are domain-bound to the contract address to prevent replay across unrelated contract contexts.  
  - Duplicate (same verifier, identity, attestation_data) is rejected.  
  - Weight is computed from attester stake (see weighted attestations).  
  - Emits `attestation_added` with (subject, id, attester, attestation_data, weight).

## Revoking attestations

- **revoke_attestation(attester, attestation_id, nonce)**  
  - Only the original verifier can revoke. Nonce consumed and incremented.  
  - The attestation is marked revoked, its ID is removed from the subject's live attestation index, and the subject attestation count is decremented with checked arithmetic.  
  - The dedup key is removed so the same triple can be attested again.  
  - Emits `attestation_revoked`.

## Queries

- **get_attestation(attestation_id)** — Returns the attestation or panics if not found.
- **get_subject_attestations(subject)** — Returns live, non-revoked attestation IDs for the identity.
- **get_subject_attestation_count(subject)** — Returns the active attestation count for the identity. It must equal `get_subject_attestations(subject).len()`.

## Security

- Verifier must be authorized and pass require_auth.
- Duplicate attestations (same verifier, identity, data) are prevented.
- Replay is prevented via per-identity nonces; see security.md.
- Attestation accounting uses checked arithmetic; overflow, underflow, or index/count drift is treated as an invariant failure instead of being silently saturated.
