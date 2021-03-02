#![cfg(feature = "runtime-benchmarks")]

use super::*;
// extern crate std;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account};
// use std::string::String;
use crate::Module as MwReconciliation;
// use crate::{Campaign, AggregatedData, ReconciledData, Kpis};

const SEED: u32 = 0;

benchmarks! {
	_ { }
	set_order {
		let n in 1 .. T::MaxBillboards::get();

		let caller = account("caller", 0, SEED);

		let mut size: u32 = 10;
		let mut creative_list = Vec::new();
		for i in 0..size {
			creative_list.push(b"video_1.m".to_vec());
		}

		let mut target_inventory = Vec::<BillboardData>::new();

		for i in 0..n {
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

	}: set_order(RawOrigin::Signed(caller), order_id.clone(), order_data_clone)
	verify {
		assert_eq!(MwReconciliation::<T>::get_order(order_id.clone()), order);
	}

	set_session_data {
		let caller: T::AccountId = account("caller", 0, SEED);

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

		let session_data = SessionData {
			id: b"SD_1".to_vec(),
			order_id: order_id.clone(),
			billboard_id: b"BB_1".to_vec(),
			creative_id: b"video_1.m".to_vec(),
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
		assert_eq!(MwReconciliation::<T>::get_verified_spots(order_date, billboard_data.id), verified_spot);
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
