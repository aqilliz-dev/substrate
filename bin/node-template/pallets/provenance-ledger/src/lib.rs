#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod benchmarking;

use frame_support::{
	weights::{Weight, Pays},
    decl_module, decl_event
};
use frame_system::{self as system, ensure_signed};

use sp_core::Hasher;
use codec::Encode;
use sp_std::prelude::*;

pub trait WeightInfo {
	fn add_activity_group() -> Weight;
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
        /// Event emitted when activity is sent.
        ActivityAdded(AccountId, Vec<u8>, Vec::<Vec<u8>>),
		ActivityGroupAdded(AccountId, Vec<u8>, u64),
    }
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		#[weight = (T::WeightInfo::add_activity_group(), Pays::No)]
		fn add_activity(origin, action_id: Vec<u8>, subject_ids: Vec::<Vec<u8>>) {
			let sender = ensure_signed(origin)?;

			let topic_provenance_ledger = T::Hashing::hash(b"provenance-ledger");
			let topic_sender = T::Hashing::hash(&sender.encode());

			let event = <T as Trait>::Event::from(RawEvent::ActivityAdded(sender, action_id, subject_ids));
			frame_system::Module::<T>::deposit_event_indexed(&[topic_provenance_ledger, topic_sender], event.into());
		}

		#[weight = (T::WeightInfo::add_activity_group(), Pays::No)]
		fn add_activity_group(origin, action_id: Vec<u8>, number_of_subject_ids: u64) {
			let sender = ensure_signed(origin)?;

			let topic_provenance_ledger = T::Hashing::hash(b"provenance-ledger-group");
			let topic_sender = T::Hashing::hash(&sender.encode());

			let event = <T as Trait>::Event::from(RawEvent::ActivityGroupAdded(sender, action_id, number_of_subject_ids));
			frame_system::Module::<T>::deposit_event_indexed(&[topic_provenance_ledger, topic_sender], event.into());
		}
	}
}
