#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account};
use crate::Module as MwReconciliation;
use crate::{helpers::*};

const SEED: u32 = 0;

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
