#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
// mod benchmarking;

use frame_support::{
	debug,
	weights::{Weight, Pays},
    decl_module, decl_event, decl_storage,
	storage::{StorageDoubleMap, StorageMap, StorageValue},
	codec::{Encode, Decode},
	sp_runtime::RuntimeDebug
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

pub type CampaignId = Vec<u8>;
pub type Platform = Vec<u8>;
pub type Source = Vec<u8>;
pub type Date = Vec<u8>;
pub type DateCampaign = Vec<u8>;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Campaign {
    name: Vec<u8>,
	total_budget: u64,
	currency: Vec<u8>,
	start_date: Date,
	end_date: Date,
	platforms: Vec::<Platform>,
	advertiser: Vec<u8>,
	brand: Vec<u8>,
	reconciliation_threshold: u64,
	version: u8,
	cpc: (bool, u64),
	cpm: (bool, u64),
	cpl: (bool, u64),
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct AggregatedData {
	campaign_id: CampaignId,
	platform: Platform,
	date: Date,
	source: Source,
	impressions: u64,
	clicks: u64,
	conversions: u64,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Kpis {
	sources: Vec::<(Platform, u64)>,
	final_count: u64,
	cost: u64,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct ReconciledData {
	amount_spent: u64,
	budget_utilisation: u64,
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


decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

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

		// let reconciled_campaign_dates_1 = <ReconciledCampaignDates>::get(&aggregated_data.campaign_id, &aggregated_data.date);
		// let campaign_date_exists = <ReconciledData>::contains_key(&campaign_date);
		// let reconciled_campaign_dates_2 = <ReconciledCampaignDates>::get(&aggregated_data.campaign_id, b"-".to_vec());
		// let reconciled_campaign_dates_3 = <ReconciledCampaignDates>::get(b"-".to_vec(), b"-".to_vec());

		// debug::info!("Reconciled data 1 {:?}", reconciled_campaign_dates_1);
		// debug::info!("Reconciled data 1 {:?}", reconciled_campaign_dates_1);
		// debug::info!("Reconciled data 2 {:?}", reconciled_campaign_dates_2);
		// debug::info!("Reconciled data 3 {:?}", reconciled_campaign_dates_3);
		// let source
		// let impressions = Kpis {

		// };

		// let clicks = Kpis {

		// };

		// let conversions = Kpis {

		// };

		// let data = ReconciledData {
		// 	amount_spent: 0,
		// 	budget_utilisation: 0,
		// 	impressions: impressions,
		// 	clicks: clicks,
		// 	conversions: conversions
		// };
	}
}
