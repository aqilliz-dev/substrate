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
use log::{info};

pub trait WeightInfo {
	fn add_activity_group() -> Weight;
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type WeightInfo: WeightInfo;
}

// This pallet's events.
// decl_event! {
//     pub enum Event<T>
// 	where
// 		AccountId = <T as system::Trait>::AccountId,
// 	{
//         /// Event emitted when activity is sent.
//         ActivityAdded(AccountId, Vec<u8>, Vec<u8>),
//     }
// }

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

// The pallet's dispatchable functions.
// decl_module! {
// 	/// The module declaration.
// 	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
// 		#[weight = 0]
// 		fn add_activity(origin, action_id: Vec<u8>, subject_ids: Vec::<Vec<u8>>) {
// 			let sender = ensure_signed(origin)?;

// 			let mut provenance_ledger = Vec::new();
// 			provenance_ledger.extend_from_slice(b"provenance-ledger");

// 			let mut slice_sender = Vec::new();
// 			slice_sender.extend_from_slice(&sender.encode()[..]);

// 			let topic_provenance_ledger = T::Hashing::hash(&provenance_ledger[..]);
// 			let topic_sender = T::Hashing::hash(&slice_sender[..]);

// 			for subject_id in &subject_ids {
// 				let sender_clone = sender.clone();
// 				let action_id_clone = action_id.clone();
// 				let subject_id_clone = subject_id.clone();

// 				let topic_subject_id = T::Hashing::hash(&subject_id[..]);

// 				// info!("🙌 =================== TOPIC PROVENANCE ==================== {:?}", topic_provenance_ledger);
// 				// info!("🙌 =================== TOPIC SENDER ======================== {:?}", topic_sender);
// 				// info!("🙌 =================== TOPIC DATA ID =============+========= {:?}", topic_data_id);

// 				let event = <T as Trait>::Event::from(RawEvent::ActivityAdded(sender_clone, action_id_clone, subject_id_clone));
// 				frame_system::Module::<T>::deposit_event_indexed(&[topic_provenance_ledger, topic_sender, topic_subject_id], event.into());
// 			}
// 		}
// 	}
// }

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		#[weight = 0]
		fn add_activity(origin, action_id: Vec<u8>, subject_ids: Vec::<Vec<u8>>) {
			let sender = ensure_signed(origin)?;

			let mut provenance_ledger = Vec::new();
			provenance_ledger.extend_from_slice(b"provenance-ledger");
			let topic_provenance_ledger = T::Hashing::hash(&provenance_ledger[..]);

			let mut slice_sender = Vec::new();
			slice_sender.extend_from_slice(&sender.encode()[..]);
			let topic_sender = T::Hashing::hash(&slice_sender[..]);

			let event = <T as Trait>::Event::from(RawEvent::ActivityAdded(sender, action_id, subject_ids));
			frame_system::Module::<T>::deposit_event_indexed(&[topic_provenance_ledger, topic_sender], event.into());
		}

		#[weight = (T::WeightInfo::add_activity_group(), Pays::No)]
		fn add_activity_group(origin, action_id: Vec<u8>, number_of_subject_ids: u64) {
			let sender = ensure_signed(origin)?;

			info!("🙌 ======================================= {:?}", T::WeightInfo::add_activity_group());

			let mut provenance_ledger = Vec::new();
			provenance_ledger.extend_from_slice(b"provenance-ledger-group");
			let topic_provenance_ledger = T::Hashing::hash(&provenance_ledger[..]);

			let mut slice_sender = Vec::new();
			slice_sender.extend_from_slice(&sender.encode()[..]);
			let topic_sender = T::Hashing::hash(&slice_sender[..]);

			let event = <T as Trait>::Event::from(RawEvent::ActivityGroupAdded(sender, action_id, number_of_subject_ids));
			frame_system::Module::<T>::deposit_event_indexed(&[topic_provenance_ledger, topic_sender], event.into());
		}
	}
}
