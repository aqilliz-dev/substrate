use crate::{mock::*};
use super::*;
use frame_support::{assert_ok, assert_noop};
use crate::{helpers::*};

#[test]
fn order_set() {
	let (order_data, order, _) = sample_data(10);

	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(
			MwReconciliation::set_order(
				Origin::signed(1),
				ORDER_ID.to_vec(),
				order_data
			)
		);

		// Read pallet storage and assert an expected result.
		assert_eq!(
			MwReconciliation::get_order(ORDER_ID.to_vec()),
			order
		);
	});
}

#[test]
fn order_set_error() {
	let (order_data, order, _) = sample_data(10);

	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_noop!(
			MwReconciliation::set_order(
				Origin::signed(1),
				ORDER_ID.to_vec(),
				order_data
			),
			Error::<Test>::InvalidInput
		);
	});
}

#[test]
fn set_session_data() {
	let (order_data, _, session_data) = sample_data(1);

	new_test_ext().execute_with(|| {
		// Set Order
		assert_ok!(
			MwReconciliation::set_order(
				Origin::signed(1),
				ORDER_ID.to_vec(),
				order_data
			)
		);

		// Set Session Data
		assert_ok!(
			MwReconciliation::set_session_data(Origin::signed(1), session_data)
		);

		let verified_spot = VerifedSpot {
			verified_audience: 1000
		};

		assert_eq!(
			MwReconciliation::get_verified_spots(
				ORDER_DATE.to_vec(),
				BILLBOARD_ID.to_vec()
			),
			verified_spot
		);
	});
}
