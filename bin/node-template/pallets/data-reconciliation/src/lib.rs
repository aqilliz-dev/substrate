#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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
	fn set_campaign() -> Weight;
	fn set_aggregated_data() -> Weight;
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type WeightInfo: WeightInfo;
}

const QUINTILLION: u128 = 1_000_000_000_000_000_000;

pub type CampaignId = Vec<u8>;
pub type Platform = Vec<u8>;
pub type Source = Vec<u8>;
pub type Date = Vec<u8>;
pub type DateCampaign = Vec<u8>;
pub type ErrorMessage = Vec<u8>;
pub type Failed = bool;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Campaign {
    name: Vec<u8>,
	total_budget: u128,
	currency: Vec<u8>,
	start_date: Date,
	end_date: Date,
	platforms: Vec<Platform>,
	advertiser: Vec<u8>,
	brand: Vec<u8>,
	reconciliation_threshold: u128,
	decimals: u32,
	version: u8,
	cpc: (bool, u128),
	cpm: (bool, u128),
	cpl: (bool, u128),
	timezone: Vec<u8>
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct AggregatedData {
	campaign_id: CampaignId,
	platform: Platform,
	date: Date,
	date_received: Date,
	source: Source,
	impressions: u128,
	clicks: u128,
	conversions: u128,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Kpis {
	final_count: u128,
	cost: u128,
	budget_utilisation: u128,
	zdmp: u128,
	platform: u128,
	client: u128
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct ReconciledData {
	amount_spent: u128,
	budget_utilisation: u128,
	clicks: Kpis,
	impressions: Kpis,
	conversions: Kpis,
}

decl_storage! {
    trait Store for Module<T: Trait> as DataReconciliation {
		/// [ID_001] -> Campaign
        Campaigns get(fn get_campaign):
            map hasher(blake2_128_concat) CampaignId => Campaign;

		/// [20201010][ID_001] -> true
        ReconciledDateCampaigns get(fn get_reconciled_date_campaigns):
            double_map hasher(blake2_128_concat) Date, hasher(blake2_128_concat) CampaignId => bool;

		/// [20201010-ID_001][Platform] -> ReconciledData
		ReconciledDataStore get(fn get_reconciled_data):
            double_map hasher(blake2_128_concat) DateCampaign, hasher(blake2_128_concat) Platform => ReconciledData;

		/// [ID_001][20201010] -> true
		ReconciledCampaignDates get(fn get_reconciled_campaign_dates):
            double_map hasher(blake2_128_concat) CampaignId, hasher(blake2_128_concat) Date => bool;
    }
}

decl_event! {
    pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
        /// Campaign is set
        CampaignSet(AccountId, CampaignId, Campaign),
		/// Aggregated Data is processed
        AggregatedDataProcessed(AccountId, AggregatedData, Failed, ErrorMessage),
    }
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		#[weight = (T::WeightInfo::set_campaign(), Pays::No)]
		fn set_campaign(origin, campaign_id: CampaignId, campaign: Campaign) {
			let sender = ensure_signed(origin)?;

			// const MAX_SENSIBLE_REASON_LENGTH: usize = 16384;
			// ensure!(reason.len() <= MAX_SENSIBLE_REASON_LENGTH, Error::<T>::ReasonTooBig);

			<Campaigns>::insert(&campaign_id, &campaign);

			// Create Event Topic name
			let topic = T::Hashing::hash(b"data-reconciliation");

			let event = <T as Trait>::Event::from(RawEvent::CampaignSet(sender, campaign_id, campaign));
			frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());
		}

		#[weight = (T::WeightInfo::set_aggregated_data(), Pays::No)]
		fn set_aggregated_data(origin, aggregated_data: AggregatedData) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Create Event Topic name
			let topic = T::Hashing::hash(b"data-reconciliation");

			let campaign_exists = <Campaigns>::contains_key(&aggregated_data.campaign_id);

			if campaign_exists {
				let campaign = <Campaigns>::get(&aggregated_data.campaign_id);
				let campaign_platforms = campaign.platforms;

				let campaign_platform_exists = if campaign_platforms.contains(&aggregated_data.platform) {
					true
				} else {
					false
				};

				if campaign_platform_exists {
					<ReconciledDateCampaigns>::insert(&aggregated_data.date, &aggregated_data.campaign_id, true);
					<ReconciledCampaignDates>::insert(&aggregated_data.campaign_id, &aggregated_data.date, true);

					let mut date_campaign = aggregated_data.date.clone();
					let campaign_id = aggregated_data.campaign_id.clone();

					date_campaign.extend(b"-".to_vec());
					date_campaign.extend(campaign_id);

					let failed = Self::update_recociled_data_record(&date_campaign, &aggregated_data);

					if failed {
						let event = <T as Trait>::Event::from(RawEvent::AggregatedDataProcessed(sender, aggregated_data, true, b"Aggregated data is not incremental".to_vec()));
						frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

						return Ok(())
					}

					let event = <T as Trait>::Event::from(RawEvent::AggregatedDataProcessed(sender, aggregated_data, false, b"".to_vec()));
					frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

					Ok(())
				} else {
					let event = <T as Trait>::Event::from(RawEvent::AggregatedDataProcessed(sender, aggregated_data, true, b"Platform does not exist for the Campaign".to_vec()));
					frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

					Ok(())
				}
			} else {
				let event = <T as Trait>::Event::from(RawEvent::AggregatedDataProcessed(sender, aggregated_data, true, b"Campaign ID does not exist".to_vec()));
				frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

				Ok(())
			}
		}
	}
}

impl<T: Trait> Module<T> {
	fn create_recociled_data_record(date_campaign: &DateCampaign, aggregated_data: &AggregatedData) {
		let mut kpis_impressions = Kpis {
			final_count: 0,
			cost: 0,
			budget_utilisation: 0,
			zdmp: 0,
			platform: 0,
			client: 0
		};

		let mut kpis_clicks = Kpis {
			final_count: 0,
			cost: 0,
			budget_utilisation: 0,
			zdmp: 0,
			platform: 0,
			client: 0
		};

		let mut kpis_conversions = Kpis {
			final_count: 0,
			cost: 0,
			budget_utilisation: 0,
			zdmp: 0,
			platform: 0,
			client: 0
		};

		Self::update_kpis(&aggregated_data, &mut kpis_impressions, &mut kpis_clicks, &mut kpis_conversions);

		kpis_impressions.final_count = *(&aggregated_data.impressions);
		kpis_clicks.final_count = *(&aggregated_data.clicks);
		kpis_conversions.final_count = *(&aggregated_data.conversions);

		let record = ReconciledData {
			amount_spent: 0,
			budget_utilisation: 0,
			impressions: kpis_impressions,
			clicks: kpis_clicks,
			conversions: kpis_conversions
		};

		<ReconciledDataStore>::insert(&date_campaign, &aggregated_data.platform, record);
	}

	fn update_recociled_data_record(date_campaign: &DateCampaign, aggregated_data: &AggregatedData) -> bool {
		let campaign_date_platform_exists = <ReconciledDataStore>::contains_key(&date_campaign, &aggregated_data.platform);

		if !campaign_date_platform_exists {
			Self::create_recociled_data_record(&date_campaign, &aggregated_data);
		}

		let mut record = <ReconciledDataStore>::get(&date_campaign, &aggregated_data.platform);

		let failed = Self::update_kpis(&aggregated_data, &mut record.impressions, &mut record.clicks, &mut record.conversions);

		if failed {
			return true
		}

		let campaign = <Campaigns>::get(&aggregated_data.campaign_id);
		let reconciliation_threshold = campaign.reconciliation_threshold;
		let total_budget = campaign.total_budget;
		let decimals = campaign.decimals;
		let (cpc_applies, cpc_val) = campaign.cpc;
		let (cpm_applies, cpm_val) = campaign.cpm;
		let (cpl_applies, cpl_val) = campaign.cpl;
		let percentage_threshold: FixedU128 = FixedU128::from_inner(reconciliation_threshold) / FixedU128::from_inner(100);

		// Clicks
		Self::run_reconciliation(&mut record.clicks, percentage_threshold);
		let (cliks_budget_utilization, clicks_cost) = if cpc_applies { Self::update_costs(&mut record.clicks, total_budget, cpc_val, decimals) } else { (0, 0) };
		// Conversions
		Self::run_reconciliation(&mut record.conversions, percentage_threshold);
		let (conversions_budget_utilization, conversions_cost) = if cpl_applies { Self::update_costs(&mut record.conversions, total_budget, cpl_val, decimals) } else { (0, 0) };
		// Impressions
		Self::run_reconciliation(&mut record.impressions, percentage_threshold);
		let (impressions_budget_utilization, impressions_cost) = if cpm_applies { Self::update_costs(&mut record.impressions, total_budget, cpm_val/1000, decimals) } else { (0, 0) };

		// Update 'budget_utilization' and 'amount_spent'
		record.budget_utilisation = cliks_budget_utilization + conversions_budget_utilization + impressions_budget_utilization;
		// record.amount_spent = Self::multiply(record.budget_utilisation, total_budget, decimals) / 100;
		record.amount_spent = clicks_cost + conversions_cost + impressions_cost;

		<ReconciledDataStore>::insert(&date_campaign, &aggregated_data.platform, record);

		return false;
	}

	fn update_kpis(
		aggregated_data: &AggregatedData,
		kpis_impressions: &mut Kpis,
		kpis_clicks: &mut Kpis,
		kpis_conversions: &mut Kpis
	) -> bool {
		if *(&aggregated_data.source) == b"zdmp".to_vec() {
			if
				aggregated_data.impressions < kpis_impressions.zdmp ||
				aggregated_data.clicks < kpis_clicks.zdmp ||
				aggregated_data.conversions < kpis_conversions.zdmp
			{
				return true
			}
			kpis_impressions.zdmp = aggregated_data.impressions;
			kpis_clicks.zdmp = aggregated_data.clicks;
			kpis_conversions.zdmp = aggregated_data.conversions;
		} else if *(&aggregated_data.source) == b"client".to_vec() {
			if
				aggregated_data.impressions < kpis_impressions.client ||
				aggregated_data.clicks < kpis_clicks.client ||
				aggregated_data.conversions < kpis_conversions.client
			{
				return true
			}
			kpis_impressions.client = aggregated_data.impressions;
			kpis_clicks.client = aggregated_data.clicks;
			kpis_conversions.client = aggregated_data.conversions;
		} else {
			if
				aggregated_data.impressions < kpis_impressions.platform ||
				aggregated_data.clicks < kpis_clicks.platform ||
				aggregated_data.conversions < kpis_conversions.platform
			{
				return true
			}
			kpis_impressions.platform = aggregated_data.impressions;
			kpis_clicks.platform = aggregated_data.clicks;
			kpis_conversions.platform = aggregated_data.conversions;
		}
		return false
	}

	fn run_reconciliation(kpi: &mut Kpis, percentage_threshold: FixedU128) {
		let count_zdmp: FixedU128 = FixedU128::from_inner(*(&kpi.zdmp)* QUINTILLION);
		let count_platform: FixedU128 = FixedU128::from_inner(*(&kpi.platform)* QUINTILLION);

		let count_zdmp_threshold = count_zdmp * percentage_threshold;
		let count_zdmp_ceil = count_zdmp + count_zdmp_threshold;
		let count_zdmp_floor = count_zdmp - count_zdmp_threshold;

		if kpi.platform != 0 && kpi.zdmp != 0 && (count_platform <= count_zdmp_ceil) && (count_platform >= count_zdmp_floor) {
			kpi.final_count = kpi.platform;
		} else {
			if kpi.zdmp != 0 {
				kpi.final_count = kpi.zdmp;
			} else if kpi.platform != 0 {
				kpi.final_count = kpi.platform;
			}
		}

		let count_final: FixedU128 = FixedU128::from_inner(*(&kpi.final_count)* QUINTILLION);
		let count_client: FixedU128 = FixedU128::from_inner(*(&kpi.client)* QUINTILLION);

		let count_final_threshold = count_final * percentage_threshold;
		let count_final_ceil = count_final + count_final_threshold;
		let count_final_floor = count_final - count_final_threshold;

		if kpi.client != 0 && kpi.final_count != 0 && (count_client <= count_final_ceil) && (count_client >= count_final_floor) {
			kpi.final_count = kpi.client;
		} else if kpi.client == 0 && kpi.zdmp == 0 && kpi.platform == 0 {
			kpi.final_count = 0;
		}
	}

	fn update_costs(
		kpi: &mut Kpis,
		total_budget: u128,
		factor: u128,
		decimals: u32
	) -> (u128, u128) {
		kpi.cost = Self::multiply(kpi.final_count * 10u128.pow(decimals), factor, decimals);
		kpi.budget_utilisation = Self::divide(kpi.cost, total_budget, decimals) * 100;
		return (kpi.budget_utilisation, kpi.cost)
	}

	pub fn divide(a: u128, b: u128, decimals: u32) -> u128 {
		let factor = 10u128.pow(decimals);
		return a * factor/ b
	}

	pub fn multiply(a: u128, b: u128, decimals: u32) -> u128 {
		let factor = 10u128.pow(decimals);
		return (a * b) / factor
	}
}
