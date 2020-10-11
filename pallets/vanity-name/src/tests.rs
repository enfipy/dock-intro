use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::{WithdrawReason, WithdrawReasons}};

#[test]
fn test_register_name() {
    new_test_ext().execute_with(|| {
        assert_ok!(VanityNameModule::register_name(
            Origin::signed(1),
            "name".as_bytes().to_vec(),
            1000000,
        ));
        assert_eq!(
            VanityNameModule::registered_names(1),
            Some(super::VanityName {
                name: "name".as_bytes().to_vec(),
                price: 2500,
                registered_until: 40001,
                owner: 1,
            })
        );
    });
}

#[test]
fn correct_error_for_name_not_registered() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            VanityNameModule::renew_registration(
                Origin::signed(1),
                "name".as_bytes().to_vec(),
                1000000,
            ),
            Error::<Test>::NameNotRegistered
        );
    });
}
 
#[test]
fn test_register_and_renew_name() {
    new_test_ext().execute_with(|| {
        assert_ok!(VanityNameModule::register_name(
            Origin::signed(1),
            "name".as_bytes().to_vec(),
            100000,
        ));
        assert_eq!(
            VanityNameModule::registered_names(1),
            Some(super::VanityName {
                name: "name".as_bytes().to_vec(),
                price: 2500,
                registered_until: 4001,
                owner: 1,
            })
		);
		let reasons: WithdrawReasons = WithdrawReason::Reserve.into();
		let lock = pallet_balances::BalanceLock{
			id: *b"name_buy",
			amount: 100000,
			reasons: reasons.into(),
		};
		assert_eq!(Balances::locks(1), vec![lock]);

        assert_ok!(VanityNameModule::renew_registration(
            Origin::signed(1),
            "name".as_bytes().to_vec(),
            100000,
        ));
        assert_eq!(
            VanityNameModule::registered_names(1),
            Some(super::VanityName {
                name: "name".as_bytes().to_vec(),
                price: 2500,
                registered_until: 8001,
                owner: 1,
            })
        );
    });
}
