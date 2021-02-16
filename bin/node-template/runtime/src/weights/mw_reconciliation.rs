//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl mw_reconciliation::WeightInfo for WeightInfo {
	fn set_order() -> Weight {
		(90_000_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	// fn set_aggregated_data() -> Weight {
	// 	(205_000_000 as Weight)
	// 		.saturating_add(DbWeight::get().reads(3 as Weight))
	// 		.saturating_add(DbWeight::get().writes(4 as Weight))
	// }
}
