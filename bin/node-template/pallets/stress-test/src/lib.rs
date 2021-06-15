#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	weights::{Pays},
	codec::{Encode, Decode},
	storage::{StorageDoubleMap},
    decl_module, decl_event, decl_storage,
	sp_runtime::{RuntimeDebug},
};
use frame_system::{self as system, ensure_signed};

use sp_std::prelude::*;
use sp_core::Hasher;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type CampaignId = Vec<u8>;
pub type AuctionId = Vec<u8>;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Input {
	campaign_id: CampaignId,
	auction_id: AuctionId,
	one: u32,
	two: u32,
	three: u32,
	four: u32,
	five: u32,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Output {
	one: u32,
	two: u32,
	three: u32,
	four: u32,
	five: u32,
}

decl_storage! {
    trait Store for Module<T: Trait> as StressTest {
        ReconciledCampaignAuctions get(fn get_imp_reconciliation):
            double_map hasher(blake2_128_concat) CampaignId, hasher(blake2_128_concat) AuctionId => Output;
    }
}

decl_event! {
    pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
        /// Event emitted when transaction is sent.
        SentTransaction(AccountId, CampaignId, AuctionId, u32, u32, u32, u32, u32),
        CampaignDeleted(AccountId, CampaignId),
    }
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		#[weight = (500_000_000, Pays::No)]
		fn send_transaction(origin, input: Input) {
			let sender = ensure_signed(origin)?;

			let Input {campaign_id, auction_id, one, two, three, four, five} = input;

			let output = Output { one, two, three, four, five };

			<ReconciledCampaignAuctions>::insert(&campaign_id, &auction_id, output);

			let mut stress_test = Vec::new();
			stress_test.extend_from_slice(b"stress-test");
			let topic_stress_test = T::Hashing::hash(&stress_test[..]);

			// let mut slice_sender = Vec::new();
			// slice_sender.extend_from_slice(&sender.encode()[..]);
			// let topic_sender = T::Hashing::hash(&slice_sender[..]);

			let event = <T as Trait>::Event::from(
				RawEvent::SentTransaction(sender, campaign_id, auction_id, one, two, three, four, five)
			);
			// frame_system::Module::<T>::deposit_event_indexed(&[topic_stress_test, topic_sender], event.into());
			frame_system::Module::<T>::deposit_event_indexed(&[topic_stress_test], event.into());
		}

		#[weight = (500_000_000, Pays::No)]
		fn delete_campaign(origin, campaign_id: CampaignId) {
			let sender = ensure_signed(origin)?;

			<ReconciledCampaignAuctions>::remove_prefix(&campaign_id);

			// Create Event Topic name
			let mut topic_name = Vec::new();
			topic_name.extend_from_slice(b"stress-test");
			let topic = T::Hashing::hash(&topic_name[..]);

			let event = <T as Trait>::Event::from(RawEvent::CampaignDeleted(sender, campaign_id));
			frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());
		}
	}
}
