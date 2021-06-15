#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account};
use crate::Module as MwReconciliation;

const SEED: u32 = 0;
const ORDER_ID: &[u8] = b"ORD_001";
const CREATIVE_ID: &[u8] = b"video_1.m";

fn sample_data(range: u32) -> (Vec<u8>, OrderData, Order) {
	let size = 10;
	let creative_list = vec![CREATIVE_ID.to_vec(); size];
	let mut target_inventory = Vec::<BillboardData>::new();

	for i in 0..range {
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

	let order_id = ORDER_ID.to_vec();

	let order_data = OrderData {
		start_date: 1614137312,
		end_date: 1614138312,
		total_spots: 800,
		total_audiences: 50000,
		creative_list,
		target_inventory,
	};

	let order = Order {
		start_date: order_data.start_date,
		end_date: order_data.end_date,
		total_spots: order_data.total_spots,
		total_audiences: order_data.total_audiences,
		creative_list: order_data.creative_list.clone()
	};

	(order_id, order_data, order)
}

benchmarks! {
	_ { }
	set_order {
		let n in 1 .. T::MaxBillboards::get();
		let caller = account("caller", 0, SEED);
		let (order_id, order_data, order) = sample_data(n);

	}: set_order(RawOrigin::Signed(caller), order_id.clone(), order_data)
	verify {
		assert_eq!(MwReconciliation::<T>::get_order(order_id.clone()), order);
	}

	set_session_data {
		let caller: T::AccountId = account("caller", 0, SEED);

		let (order_id, order_data, order) = sample_data(1);

		let session_data = SessionData {
			id: b"SD_1".to_vec(),
			order_id: ORDER_ID.to_vec(),
			billboard_id: 0_u32.to_be_bytes().to_vec(),
			creative_id: CREATIVE_ID.to_vec(),
			timestamp: 1614137313,
			date: b"20201010".to_vec(),
			duration: 10
		};

		let caller_origin: <T as frame_system::Trait>::Origin = RawOrigin::Signed(caller.clone()).into();

		MwReconciliation::<T>::set_order(caller_origin, order_id, order_data)?;

		let mut order_date = session_data.clone().order_id;
		let date = session_data.clone().date;

		order_date.extend(b"-".to_vec());
		order_date.extend(date);

		let verified_spot = VerifedSpot {
			verified_audience: 1000
		};

	}: set_session_data(RawOrigin::Signed(caller), session_data)
	verify {
		// assert_eq!(MwReconciliation::<T>::get_billboards(billboard_data.id), reconciled_data);
		assert_eq!(MwReconciliation::<T>::get_verified_spots(order_date, 0_u32.to_be_bytes().to_vec()), verified_spot);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests_composite::{ExtBuilder, Test};
	use frame_support::assert_ok;

	#[test]
	fn set_order() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(test_benchmark_set_order::<Test>());
		});
	}

	#[test]
	fn set_aggregated_data() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(test_benchmark_set_aggregated_data::<Test>());
		});
	}
}
