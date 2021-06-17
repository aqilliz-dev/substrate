use crate::{mock::*};
use super::*;
use frame_support::{assert_ok};
use crate::{helpers::*};

#[test]
fn order_set() {
	let (order_id, order_data, order) = sample_data(10);

	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(MwReconciliation::set_order(Origin::signed(1),order_id.clone(), order_data));
		// Read pallet storage and assert an expected result.
		assert_eq!(MwReconciliation::get_order(order_id.clone()), order);
	});
}

#[test]
fn set_session_data() {
	let (order_id, order_data, order) = sample_data(1);

	new_test_ext().execute_with(|| {
		// Set Order
		MwReconciliation::set_order(Origin::signed(1), order_id.clone(), order_data);

		let session_data = SessionData {
			id: b"SD_1".to_vec(),
			order_id: order_id.clone(),
			billboard_id: 0_u32.to_be_bytes().to_vec(),
			creative_id: CREATIVE_ID.to_vec(),
			timestamp: 1614137313,
			date: b"20201010".to_vec(),
			duration: 10
		};

		// Set Session Data
		MwReconciliation::set_session_data(Origin::signed(1), session_data.clone());

		let mut order_date = session_data.clone().order_id;
		let date = session_data.clone().date;

		order_date.extend(b"-".to_vec());
		order_date.extend(date);

		let verified_spot = VerifedSpot {
			verified_audience: 1000
		};

		assert_eq!(MwReconciliation::get_verified_spots(order_date, 0_u32.to_be_bytes().to_vec()), verified_spot);
	});
}
