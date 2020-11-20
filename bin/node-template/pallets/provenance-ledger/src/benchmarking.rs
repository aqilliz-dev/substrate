#![cfg(feature = "runtime-benchmarks")]

use super::*;
// extern crate std;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account};
// use std::string::String;

const SEED: u32 = 0;

benchmarks! {
	_ { }
	add_activity_group {
		let caller = account("caller", 0, SEED);

		// let action_id_string = String::from("uploaded");
		// let action_id = action_id_string.into_bytes();
		let action_id = vec![108, 111, 97, 100, 101, 100]; // "loaded"
		let number_of_subject_ids:u64 = 1000000;

	}: add_activity_group(RawOrigin::Signed(caller), action_id, number_of_subject_ids)
	verify {
		assert_eq!(true, true);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests_composite::{ExtBuilder, Test};
	use frame_support::assert_ok;

	#[test]
	fn add_activity_group() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(test_benchmark_add_activity_group::<Test>());
		});
	}
}
