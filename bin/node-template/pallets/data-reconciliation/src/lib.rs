#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
// mod benchmarking;

use frame_support::{
	debug,
	ensure,
	weights::{Weight, Pays},
    decl_module, decl_event, decl_storage, decl_error,
	storage::{StorageDoubleMap, StorageMap, StorageValue},
	codec::{Encode, Decode},
	sp_runtime::{RuntimeDebug, Percent, FixedU128}
};

use frame_system::{self as system, ensure_signed};

use sp_core::Hasher;
use sp_std::prelude::*;
use log::{info};
// use sp_runtime::RuntimeDebug;

// pub trait WeightInfo {
// 	fn add_activity_group() -> Weight;
// }

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	// type WeightInfo: WeightInfo;
}

const QUINTILLION: u128 = 1_000_000_000_000_000_000;

pub type CampaignId = Vec<u8>;
pub type Platform = Vec<u8>;
pub type Source = Vec<u8>;
pub type Date = Vec<u8>;
pub type DateCampaign = Vec<u8>;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Campaign {
    name: Vec<u8>,
	total_budget: u128,
	currency: Vec<u8>,
	start_date: Date,
	end_date: Date,
	platforms: Vec::<Platform>,
	advertiser: Vec<u8>,
	brand: Vec<u8>,
	reconciliation_threshold: u128,
	version: u8,
	cpc: (bool, u128),
	cpm: (bool, u128),
	cpl: (bool, u128),
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct AggregatedData {
	campaign_id: CampaignId,
	platform: Platform,
	date: Date,
	source: Source,
	impressions: u128,
	clicks: u128,
	conversions: u128,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Kpis {
	final_count: u128,
	cost: u128,
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
		/// Aggregated Data is set
        AggregatedDataProcessed(AccountId, DateCampaign, AggregatedData),
    }
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Campaign ID does not exist
		CampaignDoesNotExist,

		// /// The value cannot be incremented further because it has reached the maimum allowed value
		// MaxValueReached,
	}
}


decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		#[weight = (0, Pays::No)]
		fn set_campaign(origin, campaign_id: CampaignId, campaign: Campaign) {
			let sender = ensure_signed(origin)?;

			// const MAX_SENSIBLE_REASON_LENGTH: usize = 16384;
			// ensure!(reason.len() <= MAX_SENSIBLE_REASON_LENGTH, Error::<T>::ReasonTooBig);

			<Campaigns>::insert(&campaign_id, &campaign);

			// Create Event Topic name
			let mut topic_name = Vec::new();
			topic_name.extend_from_slice(b"data-reconciliation");
			let topic = T::Hashing::hash(&topic_name[..]);

			let event = <T as Trait>::Event::from(RawEvent::CampaignSet(sender, campaign_id, campaign));
			frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());
		}

		#[weight = (0, Pays::No)]
		fn set_aggregated_data(origin, aggregated_data: AggregatedData) {
			let sender = ensure_signed(origin)?;

			// ensure!(<Campaigns>::contains_key(&aggregated_data.campaign_id), Error::<T>::CampaignDoesNotExist);

			<ReconciledDateCampaigns>::insert(&aggregated_data.date, &aggregated_data.campaign_id, true);
			<ReconciledCampaignDates>::insert(&aggregated_data.campaign_id, &aggregated_data.date, true);

			let mut date_campaign = aggregated_data.date.clone();
			let campaign_id = aggregated_data.campaign_id.clone();

			date_campaign.extend(b"-".to_vec());
			date_campaign.extend(campaign_id);

			Self::calculate_reconciled_data(&date_campaign, &aggregated_data);

			// Create Event Topic name
			let mut topic_name = Vec::new();
			topic_name.extend_from_slice(b"data-reconciliation");
			let topic = T::Hashing::hash(&topic_name[..]);

			let event = <T as Trait>::Event::from(RawEvent::AggregatedDataProcessed(sender, date_campaign, aggregated_data));
			frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());
		}
	}
}

impl<T: Trait> Module<T> {
	fn calculate_reconciled_data(date_campaign: &DateCampaign, aggregated_data: &AggregatedData) {

		debug::info!("Date-Campaign {:?}", *date_campaign);
		debug::info!("Aggregated data {:?}", *aggregated_data);

		let mut sources: Vec::<(Platform, u64)> = Vec::new();

		let campaign_date_platform_exists = <ReconciledDataStore>::contains_key(&date_campaign, &aggregated_data.platform);

		if campaign_date_platform_exists == false {
			Self::create_recociled_data_record(&date_campaign, &aggregated_data);
		} else {
			Self::update_recociled_data_record(&date_campaign, &aggregated_data)
		}
	}

	fn create_recociled_data_record(date_campaign: &DateCampaign, aggregated_data: &AggregatedData) {
		let mut kpis_impressions = Kpis {
			final_count: 0,
			cost: 0,
			zdmp: 0,
			platform: 0,
			client: 0
		};

		let mut kpis_clicks = Kpis {
			final_count: 0,
			cost: 0,
			zdmp: 0,
			platform: 0,
			client: 0
		};

		let mut kpis_conversions = Kpis {
			final_count: 0,
			cost: 0,
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

	fn update_recociled_data_record(date_campaign: &DateCampaign, aggregated_data: &AggregatedData) {
		let mut record = <ReconciledDataStore>::get(&date_campaign, &aggregated_data.platform);

		// let mut kpis_impressions = record.impressions;
		// let mut kpis_clicks = record.clicks;
		// let mut kpis_conversions = record.conversions;

		// Self::update_kpis(&aggregated_data, &mut kpis_impressions, &mut kpis_clicks, &mut kpis_conversions);
		Self::update_kpis(&aggregated_data, &mut record.impressions, &mut record.clicks, &mut record.conversions);

		Self::run_reconciliation(&aggregated_data, &mut record);

		<ReconciledDataStore>::insert(&date_campaign, &aggregated_data.platform, record);
	}

	fn update_kpis(
		aggregated_data: &AggregatedData,
		kpis_impressions: &mut Kpis,
		kpis_clicks: &mut Kpis,
		kpis_conversions: &mut Kpis
	) {
		if *(&aggregated_data.source) == b"zdmp".to_vec() {
			kpis_impressions.zdmp = aggregated_data.impressions;
			kpis_clicks.zdmp = aggregated_data.clicks;
			kpis_conversions.zdmp = aggregated_data.conversions;
		} else if *(&aggregated_data.source) == b"client".to_vec() {
			kpis_impressions.client = aggregated_data.impressions;
			kpis_clicks.client = aggregated_data.clicks;
			kpis_conversions.client = aggregated_data.conversions;
		} else {
			kpis_impressions.platform = aggregated_data.impressions;
			kpis_clicks.platform = aggregated_data.clicks;
			kpis_conversions.platform = aggregated_data.conversions;
		}
	}

	fn run_reconciliation(aggregated_data: &AggregatedData, record: &mut ReconciledData) {
		let campaign = <Campaigns>::get(&aggregated_data.campaign_id);
		let reconciliation_threshold = campaign.reconciliation_threshold;

		let impressions_zdmp: FixedU128 = FixedU128::from_inner(*(&record.impressions.zdmp)* QUINTILLION);
		let impressions_platform: FixedU128 = FixedU128::from_inner(*(&record.impressions.platform)* QUINTILLION);

		let percentage_threshold: FixedU128 = FixedU128::from_inner(reconciliation_threshold) / FixedU128::from_inner(100);
		let impressions_zdmp_threshold = impressions_zdmp * percentage_threshold;

		debug::info!("Percentage Threshold {:?}", percentage_threshold);
		debug::info!("Impresions ZDMP {:?}", *(&record.impressions.zdmp));
		debug::info!("Impresions ZDMP U128 {:?}", impressions_zdmp);
		debug::info!("Impressions ZDMP Threshold U128 {:?}", impressions_zdmp_threshold);

		// kpis_clicks.final_count = *(&aggregated_data.clicks);
		// kpis_conversions.final_count = *(&aggregated_data.conversions);
		// *(&record).
		// let record = ReconciledData {
		// 	amount_spent: 0,
		// 	budget_utilisation: 0,
		// 	impressions: *(&kpis_impressions),
		// 	clicks: *kpis_clicks,
		// 	conversions: *kpis_conversions
		// };
	}
}
