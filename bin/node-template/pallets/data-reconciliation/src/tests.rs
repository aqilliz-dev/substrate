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
		assert_ok!(
			DataReconciliation::set_campaign(
				Origin::signed(1),
				CAMPAIGN_ID.to_vec(),
				campaign.clone()
			)
		);

		let Campaign {
			decimals,
			cpc,
			total_budget,
			..
		} = campaign;

		let (_, cpc_value) = cpc;

		// ========================= CLICKS =================================

		//--------------------- SINGLE SOURCE - zdmp -----------------------
		//++++++++++++++
		// zdmp:     100
		// platform: 0
		// client:   0
		// total:    100
		//++++++++++++++

		// Initial Aggregated Data
		let mut aggregated_data = get_aggregated_data(ZDMP, 0, 100, 0);

		// Set Aggregated Data
		assert_ok!(
			DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		);

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
			aggregated_data.clicks * 10u128.pow(decimals),
			cpc_value,
			decimals
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
						aggregated_data.clicks * 10u128.pow(decimals),
						cpc_value,
						decimals
					),
					100 * 10u128.pow(decimals),
					decimals
				),
				total_budget,
				decimals
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
		let previous_aggregated_data = aggregated_data.clone();

		aggregated_data = get_aggregated_data(PLATFORM, 0, 120, 0);

		assert_ok!(
			DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		);

		// Reconciled Data
		let double_source_reconciled_data = DataReconciliation::get_reconciled_data(
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
		aggregated_data = get_aggregated_data(CLIENT, 0, 120, 0);

		assert_ok!(
			DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		);

		// Reconciled Data
		let triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
			DATE_CAMPAIGN.to_vec(),
			&aggregated_data.platform
		);

		// Test Total Count
		assert_eq!(
			triple_source_reconciled_data.clicks.final_count,
			previous_aggregated_data.clicks
		);

		// ============= OLD TESTS ARE INVALID SINCE WE DON NOT ACCEPT NOT INCREMENTAL VALUES =========

		// // _______________ Client IN Reconciliation Threshold ___________________
		// //++++++++++++++
		// // zdmp:     100
		// // platform: 120
		// // client:   85
		// // total:    85
		// //++++++++++++++

		// // Set Aggregated Data
		// previous_aggregated_data = aggregated_data.clone();
		// aggregated_data = get_aggregated_data(CLIENT, 0, 85, 0);

		// assert_ok!(
		// 	DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		// );

		// // Reconciled Data
		// triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
		// 	DATE_CAMPAIGN.to_vec(),
		// 	&aggregated_data.platform
		// );

		// // Test Total Count - Clicks should be incremntal
		// assert_eq!(
		// 	triple_source_reconciled_data.clicks.final_count,
		// 	previous_aggregated_data.clicks
		// );

		// // _______________ Platform IN Reconciliation Threshold ___________________
		// //++++++++++++++
		// // zdmp:     100
		// // platform: 85
		// // client:   0
		// // total:    85
		// //++++++++++++++

		// // Set Aggregated Data
		// aggregated_data = get_aggregated_data(CLIENT, 0, 0, 0);

		// assert_ok!(
		// 	DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		// );

		// aggregated_data = get_aggregated_data(PLATFORM, 0, 85, 0);

		// assert_ok!(
		// 	DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		// );

		// // Reconciled Data
		// double_source_reconciled_data = DataReconciliation::get_reconciled_data(
		// 	DATE_CAMPAIGN.to_vec(),
		// 	&aggregated_data.platform
		// );

		// // Test Total Count
		// assert_eq!(
		// 	double_source_reconciled_data.clicks.final_count,
		// 	aggregated_data.clicks
		// );

		// //--------------------- TRIPLE SOURCE - zdmp + platform + client -----------------------
		// // _______________ Client NOT in Reconciliation Threshold ___________________
		// //++++++++++++++
		// // zdmp:     100
		// // platform: 85
		// // client:   120
		// // total:    85
		// //++++++++++++++

		// // Set Aggregated Data
		// previous_aggregated_data = aggregated_data.clone();
		// aggregated_data = get_aggregated_data(CLIENT, 0, 120, 0);

		// assert_ok!(
		// 	DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		// );

		// // Reconciled Data
		// triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
		// 	DATE_CAMPAIGN.to_vec(),
		// 	&aggregated_data.platform
		// );

		// // Test Total Count
		// assert_eq!(
		// 	triple_source_reconciled_data.clicks.final_count,
		// 	previous_aggregated_data.clicks
		// );

		// // _______________ Client IN Reconciliation Threshold ___________________
		// //++++++++++++++
		// // zdmp:     100
		// // platform: 85
		// // client:   80
		// // total:    80
		// //++++++++++++++

		// // Set Aggregated Data
		// aggregated_data = get_aggregated_data(CLIENT, 0, 80, 0);

		// assert_ok!(
		// 	DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		// );

		// // Reconciled Data
		// triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
		// 	DATE_CAMPAIGN.to_vec(),
		// 	&aggregated_data.platform
		// );

		// // Test Total Count
		// assert_eq!(
		// 	triple_source_reconciled_data.clicks.final_count,
		// 	aggregated_data.clicks
		// );

		// // _______________ Remove Zdmp  ___________________
		// //++++++++++++++
		// // zdmp:     0
		// // platform: 85
		// // client:   80
		// // total:    80
		// //++++++++++++++

		// // Set Aggregated Data
		// previous_aggregated_data = aggregated_data.clone();
		// aggregated_data = get_aggregated_data(ZDMP, 0, 0, 0);

		// assert_ok!(
		// 	DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		// );

		// // Reconciled Data
		// triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
		// 	DATE_CAMPAIGN.to_vec(),
		// 	&aggregated_data.platform
		// );

		// // Test Total Count
		// assert_eq!(
		// 	triple_source_reconciled_data.clicks.final_count,
		// 	previous_aggregated_data.clicks
		// );

		// // _______________ Take out Client from Threshold  ___________________
		// //++++++++++++++
		// // zdmp:     0
		// // platform: 85
		// // client:   20
		// // total:    85
		// //++++++++++++++

		// // Set Aggregated Data
		// aggregated_data = get_aggregated_data(CLIENT, 0, 20, 0);

		// assert_ok!(
		// 	DataReconciliation::set_aggregated_data(Origin::signed(1), aggregated_data.clone())
		// );

		// // Reconciled Data
		// triple_source_reconciled_data = DataReconciliation::get_reconciled_data(
		// 	DATE_CAMPAIGN.to_vec(),
		// 	aggregated_data.clone().platform
		// );

		// // Test Total Count
		// assert_eq!(triple_source_reconciled_data.clicks.final_count, 85);
	});
}
