/// Tests for rolling-bond notice-period enforcement.
///
/// Security invariant: for a rolling bond, `withdraw` and `withdraw_bond` must
/// require `withdrawal_requested_at != 0` AND `now >= withdrawal_requested_at +
/// notice_period_duration`. `renew_if_rolling` must be a no-op once a withdrawal
/// has been requested.
#[cfg(test)]
mod test_rolling_notice {
    use soroban_sdk::{testutils::Ledger, Env};

    use crate::{CredenceBond, CredenceBondClient};

    const NOTICE: u64 = 7 * 24 * 3600; // 7 days in seconds
    const AMOUNT: i128 = 1_000;

    fn setup() -> (Env, CredenceBondClient<'static>) {
        let e = Env::default();
        e.mock_all_auths();
        let id = e.register_contract(None, CredenceBond);
        let client = CredenceBondClient::new(&e, &id);
        // Create a rolling bond with a 7-day notice period and a 30-day duration.
        client.create_bond(
            &e.ledger().address(),
            &AMOUNT,
            &(30 * 24 * 3600),
            &true,
            &NOTICE,
        );
        (e, client)
    }

    // --- withdraw path ---

    #[test]
    #[should_panic(expected = "withdrawal not requested")]
    fn withdraw_without_request_panics() {
        let (_e, client) = setup();
        client.withdraw(&100);
    }

    #[test]
    #[should_panic(expected = "notice period not elapsed")]
    fn withdraw_immediately_after_request_panics() {
        let (e, client) = setup();
        client.request_withdrawal();
        // Still at the same timestamp — notice has not elapsed.
        client.withdraw(&100);
    }

    #[test]
    #[should_panic(expected = "notice period not elapsed")]
    fn withdraw_one_second_before_notice_panics() {
        let (e, client) = setup();
        client.request_withdrawal();
        // Advance to one second before the notice window closes.
        e.ledger().with_mut(|l| l.timestamp += NOTICE - 1);
        client.withdraw(&100);
    }

    #[test]
    fn withdraw_at_exact_notice_boundary_succeeds() {
        let (e, client) = setup();
        client.request_withdrawal();
        // Advance to exactly the notice boundary (>=, so this must succeed).
        e.ledger().with_mut(|l| l.timestamp += NOTICE);
        let bond = client.withdraw(&100);
        assert_eq!(bond.bonded_amount, AMOUNT - 100);
    }

    #[test]
    fn withdraw_after_notice_succeeds() {
        let (e, client) = setup();
        client.request_withdrawal();
        e.ledger().with_mut(|l| l.timestamp += NOTICE + 1);
        let bond = client.withdraw(&AMOUNT);
        assert_eq!(bond.bonded_amount, 0);
    }

    // --- renew_if_rolling path ---

    #[test]
    fn renew_skipped_after_withdrawal_requested() {
        let (e, client) = setup();
        // Advance past the bond period so renewal would normally trigger.
        e.ledger().with_mut(|l| l.timestamp += 30 * 24 * 3600 + 1);
        client.request_withdrawal();
        let bond_before = client.get_identity_state();
        // renew_if_rolling must be a no-op because withdrawal_requested_at != 0.
        let bond_after = client.renew_if_rolling();
        assert_eq!(bond_before.bond_start, bond_after.bond_start);
        assert_ne!(bond_after.withdrawal_requested_at, 0);
    }

    #[test]
    fn renew_works_before_withdrawal_requested() {
        let (e, client) = setup();
        // Advance past the bond period with no withdrawal request.
        e.ledger().with_mut(|l| l.timestamp += 30 * 24 * 3600 + 1);
        let bond = client.renew_if_rolling();
        // bond_start should have been updated to now.
        assert_eq!(bond.bond_start, e.ledger().timestamp());
        assert_eq!(bond.withdrawal_requested_at, 0);
    }
}
