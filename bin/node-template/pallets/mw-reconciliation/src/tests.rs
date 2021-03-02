use crate::{mock::*};
use super::*;
use frame_support::{assert_ok, assert_noop};

#[test]
fn order_set() {
	let mut size: u32 = 10;
	let mut creative_list = Vec::new();
	for i in 0..size {
		creative_list.push(b"video_1.m".to_vec());
	}

	let mut target_inventory = Vec::<BillboardData>::new();

	size = 100;

	for i in 0..size {
		let id_bytes = i.to_be_bytes();

		let billboard_data = BillboardData {
			id: id_bytes.to_vec(),
			spot_duration: 10,
			spots_per_hour: 100,
			total_spots: 700,
			imp_multiplier_per_day: i
		};
		target_inventory.push(billboard_data);
	}

	let order_id = b"ORD_001".to_vec();
	let order_id_clone = order_id.clone();

	let order_data = OrderData {
		start_date: 1614137312,
		end_date: 1614138312,
		total_spots: 800,
		total_audiences: 50000,
		creative_list,
		target_inventory,
	};


	let order_data_clone = order_data.clone();

	let order = Order {
		start_date: order_data.start_date,
		end_date: order_data.end_date,
		total_spots: order_data.total_spots,
		total_audiences: order_data.total_audiences,
		creative_list: order_data.creative_list
	};

	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(MwReconciliation::set_order(Origin::signed(1),order_id, order_data_clone));
		// Read pallet storage and assert an expected result.
		assert_eq!(MwReconciliation::get_order(order_id_clone), order);
	});
}

#[test]
fn set_session_data() {
	let mut creative_list = Vec::new();
	creative_list.push(b"video_1.m".to_vec());

	let mut target_inventory = Vec::<BillboardData>::new();

	let billboard_data = BillboardData {
		id: b"BB_1".to_vec(),
		spot_duration: 10,
		spots_per_hour: 100,
		total_spots: 700,
		imp_multiplier_per_day: 1000
	};

	target_inventory.push(billboard_data.clone());

	let order_id = b"ORD_001".to_vec();

	let order_data = OrderData {
		start_date: 1614137312,
		end_date: 1614138312,
		total_spots: 800,
		total_audiences: 50000,
		creative_list,
		target_inventory,
	};

	new_test_ext().execute_with(|| {
		// Set Order
		MwReconciliation::set_order(Origin::signed(1), order_id.clone(), order_data.clone());

		let session_data = SessionData {
			id: b"SD_1".to_vec(),
			order_id: order_id.clone(),
			billboard_id: b"BB_1".to_vec(),
			creative_id: b"video_1.m".to_vec(),
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

		assert_eq!(MwReconciliation::get_verified_spots(order_date, billboard_data.id), verified_spot);
	});
}
