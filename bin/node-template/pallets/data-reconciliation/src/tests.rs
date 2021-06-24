use crate::{mock::*};
use super::*;
use crate::{helpers::*};
use frame_support::{assert_ok};

#[test]
fn campaign_set() {
	let campaign = get_campaign(10);

	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(
			DataReconciliation::set_campaign(
				Origin::signed(1),
				CAMPAIGN_ID.to_vec(),
				campaign.clone()
			)
		);
		// Read pallet storage and assert an expected result.
		assert_eq!(
			DataReconciliation::get_campaign(CAMPAIGN_ID.to_vec()),
			campaign
		);
	});
}

#[test]
fn set_aggregated_data_platform_in() {
	let campaign = get_campaign(10);

	new_test_ext().execute_with(|| {
		// Set Campaign
		DataReconciliation::set_campaign(
			Origin::signed(1),
			CAMPAIGN_ID.to_vec(),
			campaign.clone()
		);

		let (_, cpc) = campaign.cpc;

		// Initial Aggregated Data
		let mut aggregated_data = get_aggregated_data();

		// ========================= CLICKS =================================

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
		let single_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform
		);

		// Test Total Count
		assert_eq!(
			single_source_reconciled_data.clicks.final_count,
			aggregated_data.clicks
		);

		// Test Total Cost
		let total_cost = DataReconciliation::multiply(
			aggregated_data.clicks * 10u128.pow(campaign.decimals),
			cpc,
			campaign.decimals
		);

		assert_eq!(
			single_source_reconciled_data.amount_spent,
			total_cost
		);

		// Test Budget Utilisation
		assert_eq!(
			single_source_reconciled_data.budget_utilisation,
			DataReconciliation::divide(
				DataReconciliation::multiply(
					DataReconciliation::multiply(
						aggregated_data.clicks * 10u128.pow(campaign.decimals),
						cpc,
						campaign.decimals
					),
					100 * 10u128.pow(campaign.decimals),
					campaign.decimals
				),
				campaign.total_budget,
				campaign.decimals
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
		aggregated_data.source = PLATFORM.to_vec();
		aggregated_data.clicks = 120;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		let mut double_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform);

		// Test Total Count
		assert_eq!(
			double_source_reconciled_data.clicks.final_count,
			previous_aggregated_data.clicks
		);

		//--------------------- TRIPLE SOURCE - zdmp + platform + client -----------------------
		// _______________ Client NOT in Reconciliation Threshold ___________________
		//++++++++++++++
		// zdmp:     100
		// platform: 120
		// client:   120
		// total:    100
		//++++++++++++++

		// Set Aggregated Data
		aggregated_data.source = CLIENT.to_vec();
		aggregated_data.clicks = 120;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		let mut triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform
		);

		// Test Total Count
		assert_eq!(
			triple_source_reconciled_data.clicks.final_count,
			previous_aggregated_data.clicks
		);

		// _______________ Client IN Reconciliation Threshold ___________________
		//++++++++++++++
		// zdmp:     100
		// platform: 120
		// client:   85
		// total:    85
		//++++++++++++++

		// Set Aggregated Data
		aggregated_data.source = CLIENT.to_vec();
		aggregated_data.clicks = 85;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform
		);

		// Test Total Count
		assert_eq!(
			triple_source_reconciled_data.clicks.final_count,
			aggregated_data.clicks
		);

		// _______________ Platform IN Reconciliation Threshold ___________________
		//++++++++++++++
		// zdmp:     100
		// platform: 85
		// client:   0
		// total:    85
		//++++++++++++++

		// Set Aggregated Data
		aggregated_data.source = CLIENT.to_vec();
		aggregated_data.clicks = 0;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		aggregated_data.source = PLATFORM.to_vec();
		aggregated_data.clicks = 85;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		double_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform
		);

		// Test Total Count
		assert_eq!(
			double_source_reconciled_data.clicks.final_count,
			aggregated_data.clicks
		);

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
		aggregated_data.source = CLIENT.to_vec();
		aggregated_data.clicks = 120;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform
		);

		// Test Total Count
		assert_eq!(
			triple_source_reconciled_data.clicks.final_count,
			previous_aggregated_data.clicks
		);

		// _______________ Client IN Reconciliation Threshold ___________________
		//++++++++++++++
		// zdmp:     100
		// platform: 85
		// client:   80
		// total:    80
		//++++++++++++++

		// Set Aggregated Data
		// previous_aggregated_data = aggregated_data.clone();
		aggregated_data.source = CLIENT.to_vec();
		aggregated_data.clicks = 80;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform
		);

		// Test Total Count
		assert_eq!(
			triple_source_reconciled_data.clicks.final_count,
			aggregated_data.clicks
		);

		// _______________ Remove Zdmp  ___________________
		//++++++++++++++
		// zdmp:     0
		// platform: 85
		// client:   80
		// total:    80
		//++++++++++++++

		// Set Aggregated Data
		previous_aggregated_data = aggregated_data.clone();
		aggregated_data.source = ZDMP.to_vec();
		aggregated_data.clicks = 0;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform
		);

		// Test Total Count
		assert_eq!(
			triple_source_reconciled_data.clicks.final_count,
			previous_aggregated_data.clicks
		);

		// _______________ Take out Client from Threshold  ___________________
		//++++++++++++++
		// zdmp:     0
		// platform: 85
		// client:   20
		// total:    85
		//++++++++++++++

		// Set Aggregated Data
		previous_aggregated_data = aggregated_data.clone();
		aggregated_data.source = CLIENT.to_vec();
		aggregated_data.clicks = 20;

		DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone());

		// Reconciled Data
		triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			aggregated_data.clone().platform
		);

		// Test Total Count
		assert_eq!(triple_source_reconciled_data.clicks.final_count, 85);
	});
}
