#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account};
use crate::Module as DataReconciliation;
use crate::{helpers::*};

const SEED: u32 = 0;

benchmarks! {
	_ { }
	set_campaign {
		let caller = account("caller", 0, SEED);

		let campaign = get_campaign(10);

	}: set_campaign(RawOrigin::Signed(caller), CAMPAIGN_ID.to_vec(), campaign.clone())
	verify {
		assert_eq!(
			DataReconciliation::<T>::get_campaign(CAMPAIGN_ID.to_vec()), campaign
		);
	}

	set_aggregated_data {
		let caller: T::AccountId = account("caller", 0, SEED);

		let campaign = get_campaign(10);
		let aggregated_data = get_aggregated_data(ZDMP, 100000, 30, 3);

		let kpis_clicks = Kpis {
			final_count: 30,
			cost: 21000000,
			budget_utilisation: 420000,
			zdmp: 30,
			platform: 0,
			client: 0
		};

		let kpis_impressions = Kpis {
			final_count: 100000,
			cost: 200000000,
			budget_utilisation: 4000000,
			zdmp: 100000,
			platform: 0,
			client: 0
		};

		let kpis_conversions = Kpis {
			final_count: 3,
			cost: 4200000,
			budget_utilisation: 84000,
			zdmp: 3,
			platform: 0,
			client: 0
		};

		let reconciled_data = ReconciledData {
			amount_spent: 225200000,
			budget_utilisation: 4504000,
			clicks: kpis_clicks,
			impressions: kpis_impressions,
			conversions: kpis_conversions,
		};

		let caller_origin: <T as frame_system::Trait>::Origin = RawOrigin::Signed(caller.clone()).into();

		DataReconciliation::<T>::set_campaign(caller_origin, CAMPAIGN_ID.to_vec(), campaign)?;

	}: set_aggregated_data(RawOrigin::Signed(caller), aggregated_data.clone())
	verify {
		assert_eq!(
			DataReconciliation::<T>::get_reconciled_data(
				DATE_CAMPAIGN.to_vec(),
				&aggregated_data.platform),
			reconciled_data
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests_composite::{ExtBuilder, Test};
	use frame_support::assert_ok;

	#[test]
	fn set_campaign() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(test_benchmark_set_campaign::<Test>());
		});
	}

	#[test]
	fn set_aggregated_data() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(test_benchmark_set_aggregated_data::<Test>());
		});
	}
}
