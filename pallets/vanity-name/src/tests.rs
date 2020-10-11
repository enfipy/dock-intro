use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(VanityNameModule::register_name(
			Origin::signed(1),
			"name".as_bytes().to_vec(),
			1000000,
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(VanityNameModule::registered_names(1), Some(super::VanityName {
			name: "name".as_bytes().to_vec(),
			price: 2500,
			registered_until: 401,
			owner: 1,
		}));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			VanityNameModule::renew_registration(Origin::signed(1)),
			Error::<Test>::NoneValue
		);
	});
}
