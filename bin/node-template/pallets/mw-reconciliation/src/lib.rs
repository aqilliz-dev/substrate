#![cfg_attr(not(feature = "std"), no_std)]

// #[macro_use]
// mod benchmarking;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

use frame_support::{
	weights::{Weight, Pays},
    decl_module, decl_event, decl_storage,
	storage::{StorageDoubleMap, StorageMap},
	codec::{Encode, Decode},
	sp_runtime::{RuntimeDebug, FixedU128},
	dispatch::DispatchResult
};

use frame_system::{self as system, ensure_signed};

use sp_core::Hasher;
use sp_std::prelude::*;

pub trait WeightInfo {
	fn set_order() -> Weight;
	// fn set_aggregated_data() -> Weight;
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type WeightInfo: WeightInfo;
}

const QUINTILLION: u128 = 1_000_000_000_000_000_000;

pub type OrderId = Vec<u8>;
pub type SessionId = Vec<u8>;
pub type BillboardId = Vec<u8>;
pub type CreativeId = Vec<u8>;
pub type ErrorMessage = Vec<u8>;
pub type Failed = bool;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Order {
    order_id: OrderId,
	start_date: u32,
	end_date: u32,
	total_spots: u32,
	total_audiences: u32
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct SessionData {
	session_id: SessionId,
	order_id: OrderId,
	billboard_id: BillboardId,
	creative_id: CreativeId,
	timestamp: u32,
	duration: u32
}

decl_storage! {
    trait Store for Module<T: Trait> as MwReconciliation {
		/// [ID_001] -> Order
        Orders get(fn get_order):
            map hasher(blake2_128_concat) OrderId => Order;

		// /// [20201010][ID_001] -> true
        // ReconciledDateCampaigns get(fn get_reconciled_date_campaigns):
        //     double_map hasher(blake2_128_concat) Date, hasher(blake2_128_concat) CampaignId => bool;

		// /// [20201010-ID_001][Platform] -> ReconciledData
		// ReconciledDataStore get(fn get_reconciled_data):
        //     double_map hasher(blake2_128_concat) DateCampaign, hasher(blake2_128_concat) Platform => ReconciledData;

		// /// [ID_001][20201010] -> true
		// ReconciledCampaignDates get(fn get_reconciled_campaign_dates):
        //     double_map hasher(blake2_128_concat) CampaignId, hasher(blake2_128_concat) Date => bool;
    }
}

decl_event! {
    pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
        /// Order is set
        OrderSet(AccountId, OrderId, Order),
		// /// Set Data is processed
        // SessionDataProcessed(AccountId, SessionData, Failed, ErrorMessage),
    }
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// type Error = Error<T>;

		#[weight = (T::WeightInfo::set_order(), Pays::No)]
		fn set_order(origin, order_id: OrderId, order: Order) {
			let sender = ensure_signed(origin)?;

			// const MAX_SENSIBLE_REASON_LENGTH: usize = 16384;
			// ensure!(reason.len() <= MAX_SENSIBLE_REASON_LENGTH, Error::<T>::ReasonTooBig);

			<Orders>::insert(&order_id, &order);

			// Create Event Topic name
			let mut topic_name = Vec::new();
			topic_name.extend_from_slice(b"mw-reconciliation");
			let topic = T::Hashing::hash(&topic_name[..]);

			let event = <T as Trait>::Event::from(RawEvent::OrderSet(sender, order_id, order));
			frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());
		}

		// #[weight = (T::WeightInfo::set_session_data(), Pays::No)]
		// fn set_session_data(origin, session_data: SessionData) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;

		// 	// Create Event Topic name
		// 	let mut topic_name = Vec::new();
		// 	topic_name.extend_from_slice(b"mw-reconciliation");
		// 	let topic = T::Hashing::hash(&topic_name[..]);

		// 	let order_exists = <Orders>::contains_key(&session_data.id);

		// 	if order_exists {
		// 		let order = <Orders>::get(&session_data.id);
		// 		let order_platforms = order.platforms;
		// 	} else {
		// 		let event = <T as Trait>::Event::from(RawEvent::SessionDataProcessed(sender, aggregated_data, true, "Order ID does not exist".to_vec()));
		// 		frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

		// 		Ok(())
		// 	}
		// }
	}
}

// impl<T: Trait> Module<T> {
// 	fn create_recociled_data_record(date_campaign: &DateCampaign, aggregated_data: &AggregatedData) {
// 		let mut kpis_impressions = Kpis {
// 			final_count: 0,
// 			cost: 0,
// 			budget_utilisation: 0,
// 			zdmp: 0,
// 			platform: 0,
// 			client: 0
// 		};

// 		let mut kpis_clicks = Kpis {
// 			final_count: 0,
// 			cost: 0,
// 			budget_utilisation: 0,
// 			zdmp: 0,
// 			platform: 0,
// 			client: 0
// 		};

// 		let mut kpis_conversions = Kpis {
// 			final_count: 0,
// 			cost: 0,
// 			budget_utilisation: 0,
// 			zdmp: 0,
// 			platform: 0,
// 			client: 0
// 		};

// 		Self::update_kpis(&aggregated_data, &mut kpis_impressions, &mut kpis_clicks, &mut kpis_conversions);

// 		kpis_impressions.final_count = *(&aggregated_data.impressions);
// 		kpis_clicks.final_count = *(&aggregated_data.clicks);
// 		kpis_conversions.final_count = *(&aggregated_data.conversions);

// 		let record = ReconciledData {
// 			amount_spent: 0,
// 			budget_utilisation: 0,
// 			impressions: kpis_impressions,
// 			clicks: kpis_clicks,
// 			conversions: kpis_conversions
// 		};

// 		<ReconciledDataStore>::insert(&date_campaign, &aggregated_data.platform, record);
// 	}

// 	fn update_recociled_data_record(date_campaign: &DateCampaign, aggregated_data: &AggregatedData) -> bool {
// 		let campaign_date_platform_exists = <ReconciledDataStore>::contains_key(&date_campaign, &aggregated_data.platform);

// 		if !campaign_date_platform_exists {
// 			Self::create_recociled_data_record(&date_campaign, &aggregated_data);
// 		}

// 		let mut record = <ReconciledDataStore>::get(&date_campaign, &aggregated_data.platform);

// 		let failed = Self::update_kpis(&aggregated_data, &mut record.impressions, &mut record.clicks, &mut record.conversions);

// 		if failed {
// 			return true
// 		}

// 		let campaign = <Campaigns>::get(&aggregated_data.campaign_id);
// 		let reconciliation_threshold = campaign.reconciliation_threshold;
// 		let total_budget = campaign.total_budget;
// 		let decimals = campaign.decimals;
// 		let (cpc_applies, cpc_val) = campaign.cpc;
// 		let (cpm_applies, cpm_val) = campaign.cpm;
// 		let (cpl_applies, cpl_val) = campaign.cpl;
// 		let percentage_threshold: FixedU128 = FixedU128::from_inner(reconciliation_threshold) / FixedU128::from_inner(100);

// 		// Clicks
// 		Self::run_reconciliation(&mut record.clicks, percentage_threshold);
// 		let (cliks_budget_utilization, clicks_cost) = if cpc_applies { Self::update_costs(&mut record.clicks, total_budget, cpc_val, decimals) } else { (0, 0) };
// 		// Conversions
// 		Self::run_reconciliation(&mut record.conversions, percentage_threshold);
// 		let (conversions_budget_utilization, conversions_cost) = if cpl_applies { Self::update_costs(&mut record.conversions, total_budget, cpl_val, decimals) } else { (0, 0) };
// 		// Impressions
// 		Self::run_reconciliation(&mut record.impressions, percentage_threshold);
// 		let (impressions_budget_utilization, impressions_cost) = if cpm_applies { Self::update_costs(&mut record.impressions, total_budget, cpm_val/1000, decimals) } else { (0, 0) };

// 		// Update 'budget_utilization' and 'amount_spent'
// 		record.budget_utilisation = cliks_budget_utilization + conversions_budget_utilization + impressions_budget_utilization;
// 		// record.amount_spent = Self::multiply(record.budget_utilisation, total_budget, decimals) / 100;
// 		record.amount_spent = clicks_cost + conversions_cost + impressions_cost;

// 		<ReconciledDataStore>::insert(&date_campaign, &aggregated_data.platform, record);

// 		return false;
// 	}

// 	fn update_kpis(
// 		aggregated_data: &AggregatedData,
// 		kpis_impressions: &mut Kpis,
// 		kpis_clicks: &mut Kpis,
// 		kpis_conversions: &mut Kpis
// 	) -> bool {
// 		if *(&aggregated_data.source) == b"zdmp".to_vec() {
// 			if
// 				aggregated_data.impressions < kpis_impressions.zdmp ||
// 				aggregated_data.clicks < kpis_clicks.zdmp ||
// 				aggregated_data.conversions < kpis_conversions.zdmp
// 			{
// 				return true
// 			}
// 			kpis_impressions.zdmp = aggregated_data.impressions;
// 			kpis_clicks.zdmp = aggregated_data.clicks;
// 			kpis_conversions.zdmp = aggregated_data.conversions;
// 		} else if *(&aggregated_data.source) == b"client".to_vec() {
// 			if
// 				aggregated_data.impressions < kpis_impressions.client ||
// 				aggregated_data.clicks < kpis_clicks.client ||
// 				aggregated_data.conversions < kpis_conversions.client
// 			{
// 				return true
// 			}
// 			kpis_impressions.client = aggregated_data.impressions;
// 			kpis_clicks.client = aggregated_data.clicks;
// 			kpis_conversions.client = aggregated_data.conversions;
// 		} else {
// 			if
// 				aggregated_data.impressions < kpis_impressions.platform ||
// 				aggregated_data.clicks < kpis_clicks.platform ||
// 				aggregated_data.conversions < kpis_conversions.platform
// 			{
// 				return true
// 			}
// 			kpis_impressions.platform = aggregated_data.impressions;
// 			kpis_clicks.platform = aggregated_data.clicks;
// 			kpis_conversions.platform = aggregated_data.conversions;
// 		}
// 		return false
// 	}

// 	fn run_reconciliation(kpi: &mut Kpis, percentage_threshold: FixedU128) {
// 		let count_zdmp: FixedU128 = FixedU128::from_inner(*(&kpi.zdmp)* QUINTILLION);
// 		let count_platform: FixedU128 = FixedU128::from_inner(*(&kpi.platform)* QUINTILLION);

// 		let count_zdmp_threshold = count_zdmp * percentage_threshold;
// 		let count_zdmp_ceil = count_zdmp + count_zdmp_threshold;
// 		let count_zdmp_floor = count_zdmp - count_zdmp_threshold;

// 		if kpi.platform != 0 && kpi.zdmp != 0 && (count_platform <= count_zdmp_ceil) && (count_platform >= count_zdmp_floor) {
// 			kpi.final_count = kpi.platform;
// 		} else {
// 			if kpi.zdmp != 0 {
// 				kpi.final_count = kpi.zdmp;
// 			} else if kpi.platform != 0 {
// 				kpi.final_count = kpi.platform;
// 			}
// 		}

// 		let count_final: FixedU128 = FixedU128::from_inner(*(&kpi.final_count)* QUINTILLION);
// 		let count_client: FixedU128 = FixedU128::from_inner(*(&kpi.client)* QUINTILLION);

// 		let count_final_threshold = count_final * percentage_threshold;
// 		let count_final_ceil = count_final + count_final_threshold;
// 		let count_final_floor = count_final - count_final_threshold;

// 		if kpi.client != 0 && kpi.final_count != 0 && (count_client <= count_final_ceil) && (count_client >= count_final_floor) {
// 			kpi.final_count = kpi.client;
// 		} else if kpi.client == 0 && kpi.zdmp == 0 && kpi.platform == 0 {
// 			kpi.final_count = 0;
// 		}
// 	}

// 	fn update_costs(
// 		kpi: &mut Kpis,
// 		total_budget: u128,
// 		factor: u128,
// 		decimals: u32
// 	) -> (u128, u128) {
// 		kpi.cost = Self::multiply(kpi.final_count * 10u128.pow(decimals), factor, decimals);
// 		kpi.budget_utilisation = Self::divide(kpi.cost, total_budget, decimals) * 100;
// 		return (kpi.budget_utilisation, kpi.cost)
// 	}

// 	pub fn divide(a: u128, b: u128, decimals: u32) -> u128 {
// 		let factor = 10u128.pow(decimals);
// 		return a * factor/ b
// 	}

// 	pub fn multiply(a: u128, b: u128, decimals: u32) -> u128 {
// 		let factor = 10u128.pow(decimals);
// 		return (a * b) / factor
// 	}
// }
