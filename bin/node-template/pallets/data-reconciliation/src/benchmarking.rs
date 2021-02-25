#![cfg(feature = "runtime-benchmarks")]

use super::*;
// extern crate std;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account};
// use std::string::String;
use crate::Module as DataReconciliation;
// use crate::{Campaign, AggregatedData, ReconciledData, Kpis};

const SEED: u32 = 0;

benchmarks! {
	_ { }
	set_campaign {
		let caller = account("caller", 0, SEED);

		let size = 13;
		let mut platforms_vec = Vec::new();
		for i in 0..size {
			platforms_vec.push(b"facebook".to_vec());
		}

		let campaign_id = b"ID_001".to_vec();
		let campaign_id_clone = campaign_id.clone();

		let campaign = Campaign {
			name: b"Coca Cola".to_vec(),
			total_budget: 5000000000,
			currency: b"SGD".to_vec(),
			start_date: b"20201010".to_vec(),
			end_date: b"20201111".to_vec(),
			platforms: platforms_vec,
			advertiser: b"Coca Cola Inc.".to_vec(),
			brand: b"Coke".to_vec(),
			reconciliation_threshold: 15,
			decimals: 6,
			version: 1,
			cpc: (true, 700000),
			cpm: (true, 2000000),
			cpl: (true, 1400000),
			timezone: b"timezone".to_vec(),
		};

		let campaign_clone = campaign.clone();

	}: set_campaign(RawOrigin::Signed(caller), campaign_id, campaign)
	verify {
		assert_eq!(DataReconciliation::<T>::get_campaign(campaign_id_clone), campaign_clone);
	}

	set_aggregated_data {
		let caller: T::AccountId = account("caller", 0, SEED);

		let size = 13;
		let mut platforms_vec = Vec::new();
		for i in 0..size {
			platforms_vec.push(b"facebook".to_vec());
		}

		let campaign_id = b"ID_001".to_vec();
		let campaign_id_clone = campaign_id.clone();

		let campaign = Campaign {
			name: b"Coca Cola".to_vec(),
			total_budget: 5000000000,
			currency: b"SGD".to_vec(),
			start_date: b"20201010".to_vec(),
			end_date: b"20201111".to_vec(),
			platforms: platforms_vec,
			advertiser: b"Coca Cola Inc.".to_vec(),
			brand: b"Coke".to_vec(),
			reconciliation_threshold: 15,
			decimals: 6,
			version: 1,
			cpc: (true, 700000),
			cpm: (true, 2000000),
			cpl: (true, 1400000),
			timezone: b"timezone".to_vec(),
		};

		let campaign_clone = campaign.clone();

		let aggregated_data = AggregatedData {
			campaign_id: b"ID_001".to_vec(),
			platform: b"facebook".to_vec(),
			date: b"20201010".to_vec(),
			date_received: b"20201111".to_vec(),
			source: b"zdmp".to_vec(),
			impressions: 100000,
			clicks: 30,
			conversions: 3,
		};

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

		DataReconciliation::<T>::set_campaign(caller_origin, campaign_id, campaign)?;

		let date_campaign = b"20201010-ID_001".to_vec();
		let platform = aggregated_data.platform.clone();

	}: set_aggregated_data(RawOrigin::Signed(caller), aggregated_data)
	verify {
		assert_eq!(DataReconciliation::<T>::get_reconciled_data(date_campaign, platform), reconciled_data);
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
