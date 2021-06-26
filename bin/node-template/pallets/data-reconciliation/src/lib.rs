#![cfg_attr(not(feature = "std"), no_std)]
#![feature(bool_to_option)]

#[macro_use]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod helpers;

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
const DASH: &[u8] = b"-";
const TOPIC: &[u8] = b"data-reconcilation";
const CAMPAIGN_ERROR: &[u8] = b"Campaign ID does not exist";
const PLATFORM_ERROR: &[u8] = b"Platform does not exist for the Campaign";
const DATE_ERROR: &[u8] = b"Date does not exist for that Campaign and Platform";
const DATA_ERROR: &[u8] = b"Aggregated data is not incremental";

const CLIENT: &[u8] = b"client";
const ZDMP: &[u8] = b"zdmp";
const PLATFORM: &[u8] = b"platform";

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
			let topic = T::Hashing::hash(TOPIC);

			let event = <T as Trait>::Event::from(
				RawEvent::CampaignSet(sender, campaign_id, campaign)
			);

			frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());
		}

		#[weight = (T::WeightInfo::set_aggregated_data(), Pays::No)]
		fn set_aggregated_data(origin, aggregated_data: AggregatedData) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Create Event Topic name
			let topic = T::Hashing::hash(TOPIC);
			let mut message = b"".to_vec();
			let mut failed = false;

			match Self::check_validity(&aggregated_data) {
				Err(e) => {
					if e == DATE_ERROR.to_vec() {
						Self::create_recociled_data_record(&aggregated_data);
						<ReconciledDateCampaigns>::insert(
							&aggregated_data.date,
							&aggregated_data.campaign_id,
							true
						);
						<ReconciledCampaignDates>::insert(
							&aggregated_data.campaign_id,
							&aggregated_data.date,
							true
						);
					} else {
						failed = true;
						message = e;
					}
				},
				Ok(campaign) => {
					match Self::update_recociled_data_record(&campaign, &aggregated_data) {
						Err(e) => {
							failed = true;
							message = e;
						},
						Ok(_) => {}
					}
				}
			}

			let event = <T as Trait>::Event::from(
				RawEvent::AggregatedDataProcessed(sender, aggregated_data, failed, message)
			);

			frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn get_date_campaign(aggregated_data: &AggregatedData) -> Vec<u8> {
		[&aggregated_data.date[..], DASH, &aggregated_data.campaign_id[..]].concat()
	}

	fn campaign_exists(
		aggregated_data: &AggregatedData
	) -> Result<Campaign, ErrorMessage> {
		let campaign_id = &aggregated_data.campaign_id;
		let campaign = <Campaigns>::contains_key(campaign_id);
		campaign.then_some(Self::get_campaign(campaign_id))
			.ok_or(CAMPAIGN_ERROR.to_vec())
	}

	fn platform_exists(
		aggregated_data: &AggregatedData,
		campaign: &Campaign
	) -> Result<(), ErrorMessage> {
		let platform = campaign.platforms.contains(&aggregated_data.platform);
		platform.then_some(())
			.ok_or(PLATFORM_ERROR.to_vec())
	}

	fn date_campaign_platform_exists(
		aggregated_data: &AggregatedData
	) -> Result<(), ErrorMessage> {
		let date_campaign = Self::get_date_campaign(aggregated_data);

		let date_campaign_platform = <ReconciledDataStore>::contains_key(
			&date_campaign,
			&aggregated_data.platform
		);

		date_campaign_platform.then_some(())
			.ok_or(DATE_ERROR.to_vec())
	}

	fn check_validity(
		aggregated_data: &AggregatedData
	) -> Result<Campaign, ErrorMessage> {
		let campaign = Self::campaign_exists(aggregated_data)?;
		Self::platform_exists(aggregated_data, &campaign)?;
		Self::date_campaign_platform_exists(aggregated_data)?;
		Ok(campaign)
	}

	fn create_recociled_data_record(aggregated_data: &AggregatedData) {
		let date_campaign = Self::get_date_campaign(aggregated_data);

		let mut impressions = Kpis::default();
		let mut clicks = Kpis::default();
		let mut conversions = Kpis::default();

		let _ = Self::update_kpis(
			&aggregated_data,
			&mut impressions,
			&mut clicks,
			&mut conversions
		);

		let record = ReconciledData {
			amount_spent: 0,
			budget_utilisation: 0,
			impressions,
			clicks,
			conversions
		};

		<ReconciledDataStore>::insert(&date_campaign, &aggregated_data.platform, record);

		let campaign = Self::get_campaign(&aggregated_data.campaign_id);

		let _ = Self::update_recociled_data_record(&campaign, &aggregated_data);
	}

	fn update_recociled_data_record(
		campaign: &Campaign,
		aggregated_data: &AggregatedData
	) -> Result<(), ErrorMessage> {
		let date_campaign = Self::get_date_campaign(aggregated_data);
		let mut record = <ReconciledDataStore>::get(&date_campaign, &aggregated_data.platform);

		Self::update_kpis(
			&aggregated_data,
			&mut record.impressions,
			&mut record.clicks,
			&mut record.conversions)?;

		let Campaign {
			reconciliation_threshold,
			total_budget,
			decimals,
			cpc, cpm, cpl,
			..
		} = *campaign;

		let percentage_threshold: FixedU128 = FixedU128::from_inner(reconciliation_threshold) / FixedU128::from_inner(100);
		let mut total_budget_utilisation = 0;
		let mut total_cost = 0;

		let mut kpis = [
			(&mut record.clicks, cpc),
			(&mut record.conversions, cpl),
			(&mut record.impressions, cpm)
		];

		for kpi in kpis.iter_mut() {
			let (result, (applies, value)) = kpi;
			Self::run_reconciliation(result, &percentage_threshold);

			let (budget_utilisation, cost) = if *applies {
				Self::update_costs(result, total_budget, *value, decimals)
			} else { (0, 0) };

			total_budget_utilisation += budget_utilisation;
			total_cost += cost;
		}

		record.budget_utilisation = total_budget_utilisation;
		record.amount_spent = total_cost;

		<ReconciledDataStore>::insert(&date_campaign, &aggregated_data.platform, record);

		Ok(())
	}

	fn update_kpis(
		aggregated_data: &AggregatedData,
		kpis_impressions: &mut Kpis,
		kpis_clicks: &mut Kpis,
		kpis_conversions: &mut Kpis
	) -> Result<(), ErrorMessage> {
		let AggregatedData {
			source,
			impressions,
			clicks,
			conversions,
			..
		} = aggregated_data;

		let zdpm_data = (
			ZDMP.to_vec(),
			(kpis_impressions.zdmp, kpis_clicks.zdmp, kpis_conversions.zdmp)
		);
		let client_data = (
			CLIENT.to_vec(),
			(kpis_impressions.client, kpis_clicks.client, kpis_conversions.client)
		);
		let platform_data = (
			PLATFORM.to_vec(),
			(kpis_impressions.platform, kpis_clicks.platform, kpis_conversions.platform)
		);

		let mut kpis_collection = [&zdpm_data, &client_data, &platform_data];

		for kpi in kpis_collection.iter_mut() {
			let (kpi_source, (kpi_impressions, kpi_clicks, kpi_conversions)) = kpi;

			if *source == *kpi_source {
				if
					*impressions < *kpi_impressions ||
					*clicks < *kpi_clicks ||
					*conversions < *kpi_conversions
				{
					return Err(DATA_ERROR.to_vec())
				}

				if *source == ZDMP.to_vec() {
					kpis_impressions.zdmp = *impressions;
					kpis_clicks.zdmp = *clicks;
					kpis_conversions.zdmp = *conversions;
				} else if *source == CLIENT.to_vec() {
					kpis_impressions.client = *impressions;
					kpis_clicks.client = *clicks;
					kpis_conversions.client = *conversions;
				} else if *source == PLATFORM.to_vec() {
					kpis_impressions.platform = *impressions;
					kpis_clicks.platform = *clicks;
					kpis_conversions.platform = *conversions;
				}
			}
		}

		return Ok(())
	}

	fn run_reconciliation(kpi: &mut Kpis, percentage_threshold: &FixedU128) {
		let count_zdmp: FixedU128 = FixedU128::from_inner(kpi.zdmp * QUINTILLION);
		let count_platform: FixedU128 = FixedU128::from_inner(kpi.platform * QUINTILLION);

		let count_zdmp_threshold = count_zdmp * *percentage_threshold;
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

		let count_final: FixedU128 = FixedU128::from_inner(kpi.final_count * QUINTILLION);
		let count_client: FixedU128 = FixedU128::from_inner(kpi.client * QUINTILLION);

		let count_final_threshold = count_final * *percentage_threshold;
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
