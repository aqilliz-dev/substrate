use crate::{mock::*};
use super::*;
// use crate::Module as DataReconciliation;
use frame_support::{assert_ok, assert_noop};

#[test]
fn campaign_set() {
	let campaign_id = b"ID_001".to_vec();
	let campaign_id_clone = campaign_id.clone();

	let size = 13;
	let mut platforms_vec = Vec::new();
	for i in 0..size {
		platforms_vec.push(b"facebook".to_vec());
	}

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

	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(DataReconciliation::set_campaign(Origin::signed(1),campaign_id, campaign));
		// Read pallet storage and assert an expected result.
		assert_eq!(DataReconciliation::get_campaign(campaign_id_clone), campaign_clone);
	});
}

#[test]
fn set_aggregated_data_platform_in() {
	let campaign_id = b"ID_001".to_vec();
	let campaign_id_clone = campaign_id.clone();

	let size = 13;
	let mut platforms_vec = Vec::new();
	for i in 0..size {
		platforms_vec.push(b"facebook".to_vec());
	}

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

	let date_campaign = b"20201010-ID_001".to_vec();

	new_test_ext().execute_with(|| {
		// Set Campaign
		DataReconciliation::set_campaign(Origin::signed(1),campaign_id, campaign.clone());
		let (exists, cpc) = campaign.clone().cpc;

		// Initial Aggregated Data
		let mut aggregated_data = AggregatedData {
			campaign_id: b"ID_001".to_vec(),
			platform: b"facebook".to_vec(),
			date: b"20201010".to_vec(),
			date_received: b"20201111".to_vec(),
			source: b"zdmp".to_vec(),
			impressions: 0,
			clicks: 100,
			conversions: 0,
		};

		// ================================================== CLICKS ==================================================================

		//--------------------- SINGLE SOURCE - zdmp -----------------------
		//++++++++++++++
		// zdmp:     100
		// platform: 0
		// client:   0
		// total:    100
		//++++++++++++++

		// Set Aggregated Data
		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		let single_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

		// Test Total Count
		assert_eq!(single_source_reconciled_data.clicks.final_count, aggregated_data.clone().clicks);

		// Test Total Cost
		let mut total_cost = DataReconciliation::multiply(aggregated_data.clone().clicks * 10u128.pow(campaign.clone().decimals), cpc, campaign.clone().decimals);

		assert_eq!(
			single_source_reconciled_data.amount_spent,
			total_cost
		);

		// Test Budget Utilisation
		assert_eq!(
			single_source_reconciled_data.budget_utilisation,
			DataReconciliation::divide(
				DataReconciliation::multiply(
					DataReconciliation::multiply(aggregated_data.clone().clicks * 10u128.pow(campaign.clone().decimals), cpc, campaign.clone().decimals),
					100 * 10u128.pow(campaign.clone().decimals),
					campaign.clone().decimals
				),
				campaign.clone().total_budget,
				campaign.clone().decimals
			)
		);

			//--------------------- DOUBLE SOURCE - zdmp + platform -----------------------
			// _______________ Platform NOT in Reconciliation Threshold ___________________
			//++++++++++++++
			// zdmp:     100
			// platform: 120
			// client:   0
			// total:    100
			//++++++++++++++

			// Set Aggregated Data
			let mut previous_aggregated_data = aggregated_data.clone();
			aggregated_data.source = b"platform".to_vec();
			aggregated_data.clicks = 120;

			DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

			// Reconciled Data
			let mut double_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

			// Test Total Count
			assert_eq!(double_source_reconciled_data.clicks.final_count, previous_aggregated_data.clicks);

				//--------------------- TRIPLE SOURCE - zdmp + platform + client -----------------------
				// _______________ Client NOT in Reconciliation Threshold ___________________
				//++++++++++++++
				// zdmp:     100
				// platform: 120
				// client:   120
				// total:    100
				//++++++++++++++

				// Set Aggregated Data
				aggregated_data.source = b"client".to_vec();
				aggregated_data.clicks = 120;

				DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

				// Reconciled Data
				let mut triple_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

				// Test Total Count
				assert_eq!(triple_source_reconciled_data.clicks.final_count, previous_aggregated_data.clicks);

				// _______________ Client IN Reconciliation Threshold ___________________
				//++++++++++++++
				// zdmp:     100
				// platform: 120
				// client:   85
				// total:    85
				//++++++++++++++

				// Set Aggregated Data
				aggregated_data.source = b"client".to_vec();
				aggregated_data.clicks = 85;

				DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

				// Reconciled Data
				triple_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

				// Test Total Count
				assert_eq!(triple_source_reconciled_data.clicks.final_count, aggregated_data.clicks);

			// _______________ Platform IN Reconciliation Threshold ___________________
			//++++++++++++++
			// zdmp:     100
			// platform: 85
			// client:   0
			// total:    85
			//++++++++++++++

			// Set Aggregated Data
			aggregated_data.source = b"client".to_vec();
			aggregated_data.clicks = 0;

			DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

			aggregated_data.source = b"platform".to_vec();
			aggregated_data.clicks = 85;

			DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

			// Reconciled Data
			double_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

			// Test Total Count
			assert_eq!(double_source_reconciled_data.clicks.final_count, aggregated_data.clicks);

				//--------------------- TRIPLE SOURCE - zdmp + platform + client -----------------------
				// _______________ Client NOT in Reconciliation Threshold ___________________
				//++++++++++++++
				// zdmp:     100
				// platform: 85
				// client:   120
				// total:    85
				//++++++++++++++

				// Set Aggregated Data
				previous_aggregated_data = aggregated_data.clone();
				aggregated_data.source = b"client".to_vec();
				aggregated_data.clicks = 120;

				DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

				// Reconciled Data
				triple_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

				// Test Total Count
				assert_eq!(triple_source_reconciled_data.clicks.final_count, previous_aggregated_data.clicks);

				// _______________ Client IN Reconciliation Threshold ___________________
				//++++++++++++++
				// zdmp:     100
				// platform: 85
				// client:   80
				// total:    80
				//++++++++++++++

				// Set Aggregated Data
				previous_aggregated_data = aggregated_data.clone();
				aggregated_data.source = b"client".to_vec();
				aggregated_data.clicks = 80;

				DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

				// Reconciled Data
				triple_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

				// Test Total Count
				assert_eq!(triple_source_reconciled_data.clicks.final_count, aggregated_data.clicks);

					// _______________ Remove Zdmp  ___________________
					//++++++++++++++
					// zdmp:     0
					// platform: 85
					// client:   80
					// total:    80
					//++++++++++++++

					// Set Aggregated Data
					previous_aggregated_data = aggregated_data.clone();
					aggregated_data.source = b"zdmp".to_vec();
					aggregated_data.clicks = 0;

					DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

					// Reconciled Data
					triple_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

					// Test Total Count
					assert_eq!(triple_source_reconciled_data.clicks.final_count, previous_aggregated_data.clicks);

					// _______________ Take out Client from Threshold  ___________________
					//++++++++++++++
					// zdmp:     0
					// platform: 85
					// client:   20
					// total:    85
					//++++++++++++++

					// Set Aggregated Data
					previous_aggregated_data = aggregated_data.clone();
					aggregated_data.source = b"client".to_vec();
					aggregated_data.clicks = 20;

					DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

					// Reconciled Data
					triple_source_reconciled_data = DataReconciliation::get_reconciled_data(date_campaign.clone(), aggregated_data.clone().platform);

					// Test Total Count
					assert_eq!(triple_source_reconciled_data.clicks.final_count, 85);
	});
}
