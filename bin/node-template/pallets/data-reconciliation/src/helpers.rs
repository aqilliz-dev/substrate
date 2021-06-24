use super::{Campaign, AggregatedData};

pub const CAMPAIGN_ID: &[u8] = b"ID_001";
pub const DATE_CAMPAIGN: &[u8] = b"20201010-ID_001";

pub fn get_campaign(size: usize) -> Campaign {
	let mut platforms_vec = Vec::new();

	for _ in 0..size {
		platforms_vec.push(b"facebook".to_vec());
	}

	Campaign {
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
	}
}

pub fn get_aggregated_data() -> AggregatedData {
	AggregatedData {
		campaign_id: b"ID_001".to_vec(),
		platform: b"facebook".to_vec(),
		date: b"20201010".to_vec(),
		date_received: b"20201111".to_vec(),
		source: b"zdmp".to_vec(),
		impressions: 0,
		clicks: 100,
		conversions: 0,
	}
}
