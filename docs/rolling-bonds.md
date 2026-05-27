# Rolling Bonds

Bonds that auto-renew at period end unless the user requests withdrawal with a notice period.

## Creation

Create with `create_bond(..., is_rolling: true, notice_period_duration: N)`. `notice_period_duration` is in seconds.

## Withdrawal Request

- **request_withdrawal()**: Marks that the user wants to withdraw. Sets `withdrawal_requested_at` to the current ledger timestamp. Emits `withdrawal_requested`. Panics if the bond is not rolling or a request is already pending.
- Withdrawal is **only allowed** after `now >= withdrawal_requested_at + notice_period_duration`. Calling `withdraw(amount)` or `withdraw_bond(identity)` before this threshold panics with `"notice period not elapsed"`.
- Calling either withdrawal entrypoint without a prior `request_withdrawal` panics with `"withdrawal not requested"`.

## Renewal

- **renew_if_rolling()**: If the bond is rolling, **no withdrawal has been requested**, and `now >= bond_start + bond_duration`, starts a new period: `bond_start = now`. Emits `bond_renewed`.
- Once `request_withdrawal` has been called (`withdrawal_requested_at != 0`), `renew_if_rolling` is a no-op — the bond will not auto-renew.
- Can be called by anyone when the period has ended and no withdrawal is pending.
- If not rolling or period not ended, no-op.

## Security

The notice window is the slashing window: it gives the protocol time to detect and respond to
misbehaviour before funds leave. The on-chain enforcement (`withdrawal_requested_at != 0` and
`now >= withdrawal_requested_at + notice_period_duration`) cannot be bypassed — both `withdraw`
and `withdraw_bond` perform the check with overflow-safe arithmetic (`checked_add`).

## Events

- **withdrawal_requested**: (identity, withdrawal_requested_at)
- **bond_renewed**: (identity, bond_start, bond_duration)

## Scoring

Rolling periods can be tracked via `bond_renewed` and `withdrawal_requested` for scoring and analytics.
