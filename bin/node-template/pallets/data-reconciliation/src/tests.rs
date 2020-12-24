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
fn set_aggregated_data() {
	// new_test_ext().execute_with(|| {
	// 	// Ensure the expected error is thrown when no value is present.
	// 	assert_noop!(
	// 		TemplateModule::cause_error(Origin::signed(1)),
	// 		Error::<Test>::NoneValue
	// 	);
	// });
}
