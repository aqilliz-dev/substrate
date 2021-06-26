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
		let (order_data, order, _) = sample_data(n);

	}: set_order(RawOrigin::Signed(caller), ORDER_ID.to_vec(), order_data)
	verify {
		assert_eq!(MwReconciliation::<T>::get_order(ORDER_ID.to_vec()), order);
	}

	set_session_data {
		let caller: T::AccountId = account("caller", 0, SEED);

		let (order_data, order, session_data) = sample_data(1);

		let caller_origin: <T as frame_system::Trait>::Origin = RawOrigin::Signed(caller.clone()).into();

		MwReconciliation::<T>::set_order(caller_origin, ORDER_ID.to_vec(), order_data)?;

		let verified_spot = VerifedSpot {
			verified_audience: 1000
		};

	}: set_session_data(RawOrigin::Signed(caller), session_data)
	verify {
		assert_eq!(
			MwReconciliation::<T>::get_verified_spots(
				ORDER_DATE.to_vec(),
				BILLBOARD_ID.to_vec()
			),
			verified_spot
		);
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
