// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Nicks Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! Nicks is an example pallet for keeping track of account names on-chain. It makes no effort to
//! create a name hierarchy, be a DNS replacement or provide reverse lookups. Furthermore, the
//! weights attached to this pallet's dispatchable functions are for demonstration purposes only and
//! have not been designed to be economically secure. Do not use this pallet as-is in production.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `set_name` - Set the associated name of an account; a small deposit is reserved if not already
//!   taken.
//! * `clear_name` - Remove an account's associated name; the deposit is returned.
//! * `kill_name` - Forcibly remove the associated name; the deposit is lost.

// #![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

// use frame_support::traits::{Currency, OnUnbalanced, ReservableCurrency};
pub use pallet::*;
// use sp_runtime::traits::{StaticLookup, Zero};
use sp_std::prelude::*;
use sp_io::hashing;
pub use hashing::{blake2_128, blake2_256, keccak_256, twox_128, twox_256, twox_64};


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Activity Added
		ActivityAdded {
			accountid : T::AccountId, 
			actionid : Vec<u8>, 
			subjectid : Vec::<Vec<u8>>
		},
		ActivityAddedHash {
			accountid : T::AccountId, 
			actionid : [u8; 32], 
			subjectid : [u8; 32]
		},
		/// Activity Group Added
		ActivityGroupAdded {
			accountid : T::AccountId,
			actionid : Vec<u8>,
			numofsubjectids : u8
		},
		ActivityGroupAddedHash {
			accountid : T::AccountId,
			actionid : [u8; 32],
			numofsubjectids : [u8; 32]
		}
		
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add PQS activity
		#[pallet::call_index(0)]
		#[pallet::weight({1})]
		pub fn add_activity(origin: OriginFor<T>, action_id: Vec<u8>, subject_ids: Vec::<Vec<u8>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mut provenance_ledger = Vec::new();
			provenance_ledger.extend_from_slice(b"provenance-ledger");
			// let topic_provenance_ledger = T::Hashing::hash(&provenance_ledger[..]);
			let topic_provenance_ledger: [u8; 32] = keccak_256(&provenance_ledger[..]);

			let mut slice_sender = Vec::new();
			slice_sender.extend_from_slice(&sender.encode()[..]);
			// let topic_sender = T::Hashing::hash(&slice_sender[..]);
			let topic_sender: [u8; 32] = keccak_256(&slice_sender[..]);

			Self::deposit_event(Event::<T>::ActivityAdded { accountid: sender.clone() , actionid: action_id, subjectid: subject_ids });
			Self::deposit_event(Event::<T>::ActivityAddedHash { accountid: sender.clone() , actionid: topic_provenance_ledger, subjectid: topic_sender });
			Ok(())
		}
		/// Add PQS activity group
		#[pallet::call_index(1)]
		#[pallet::weight({1})]
		pub fn add_activity_group(origin: OriginFor<T>, action_id: Vec<u8>, number_of_subject_ids: u8) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mut provenance_ledger = Vec::new();
			provenance_ledger.extend_from_slice(b"provenance-ledger-group");
			// let topic_provenance_ledger = T::Hashing::hash(&provenance_ledger[..]);
			let topic_provenance_ledger: [u8; 32] = keccak_256(&action_id);

			let mut slice_sender = Vec::new();
			slice_sender.extend_from_slice(&sender.encode()[..]);
			// let topic_sender = T::Hashing::hash(&slice_sender[..]);
			let topic_sender: [u8; 32] = keccak_256(&[number_of_subject_ids]);

			Self::deposit_event(Event::<T>::ActivityGroupAdded { accountid: sender.clone() , actionid: action_id, numofsubjectids: number_of_subject_ids });
			Self::deposit_event(Event::<T>::ActivityGroupAddedHash { accountid: sender.clone(), actionid: topic_provenance_ledger, numofsubjectids: topic_sender });
			Ok(())
		}

	}
}
