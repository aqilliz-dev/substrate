#![cfg_attr(not(feature = "std"), no_std)]

// #[macro_use]
// mod benchmarking;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

use frame_support::{
	debug,
	weights::{Weight, Pays},
    decl_module, decl_event, decl_storage, decl_error,
	storage::{StorageDoubleMap, StorageMap},
	codec::{Encode, Decode},
	sp_runtime::{RuntimeDebug, FixedU128},
	dispatch::{DispatchResult, DispatchError}
};

// use chrono::prelude::*;
// use chrono::{DateTime, TimeZone, Utc, NaiveDateTime};

// use core::fmt::Write;
// use heapless::String;
// use heapless::consts::*;

use frame_system::{self as system, ensure_signed};

use sp_core::Hasher;
use sp_std::prelude::*;

pub trait WeightInfo {
	fn set_order() -> Weight;
	fn set_session_data() -> Weight;
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
pub type Date = Vec<u8>;
pub type OrderDate = Vec<u8>;
pub type ErrorMessage = Vec<u8>;
pub type Failed = bool;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct BillboardData {
	id: BillboardId,
	spot_duration: u32,
	spots_per_hour: u32,
	total_spots: u32,
	imp_multiplier_per_day: u32
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Billboard {
	spot_duration: u32,
	spots_per_hour: u32,
	total_spots: u32,
	imp_multiplier_per_day: u32
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct OrderData {
	start_date: i64,
	end_date: i64,
	total_spots: u32,
	total_audiences: u32,
	creative_list: Vec::<CreativeId>,
	target_inventory: Vec::<BillboardData>
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Order {
	start_date: i64,
	end_date: i64,
	total_spots: u32,
	total_audiences: u32,
	creative_list: Vec::<CreativeId>
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct SessionData {
	id: SessionId,
	order_id: OrderId,
	billboard_id: BillboardId,
	creative_id: CreativeId,
	timestamp: i64,
	date: Date,
	duration: u32
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct VerifedSpot {
	verified_audience: u32
}

decl_storage! {
    trait Store for Module<T: Trait> as MwReconciliation {
		/// [OrderId] -> Order
        Orders get(fn get_order):
            map hasher(blake2_128_concat) OrderId => Order;

		/// [OrderId][BillboardId] -> Billboard
		Billboards get(fn get_billboards):
			double_map hasher(blake2_128_concat) OrderId, hasher(blake2_128_concat) BillboardId => Billboard;

		/// [OrderId][Date] -> OrderDate;
		OrdersDate get(fn get_orders_date):
			double_map hasher(blake2_128_concat) OrderId, hasher(blake2_128_concat) Date => OrderDate;

		/// [OrderDate][BillboardId] -> VerifiedSpot
		VerifiedSpots get(fn get_verified_spots):
			double_map hasher(blake2_128_concat) OrderDate, hasher(blake2_128_concat) BillboardId => VerifedSpot;
    }
}

decl_event! {
    pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
        /// Order is set
        OrderSet(AccountId, OrderId, OrderData),
		/// Set Data is processed
        SessionDataProcessed(AccountId, SessionData, Failed, ErrorMessage),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Incorrect timestamp.
        InvalidTimestamp,
    }
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// type Error = Error<T>;

		#[weight = (T::WeightInfo::set_order(), Pays::No)]
		fn set_order(origin, order_id: OrderId, order_data: OrderData) {
			let sender = ensure_signed(origin)?;

			// const MAX_SENSIBLE_REASON_LENGTH: usize = 16384;
			// ensure!(reason.len() <= MAX_SENSIBLE_REASON_LENGTH, Error::<T>::ReasonTooBig);
			let order_data_clone = order_data.clone();

			let order = Order {
				start_date: order_data_clone.start_date,
				end_date: order_data_clone.end_date,
				total_spots: order_data_clone.total_spots,
				total_audiences: order_data_clone.total_audiences,
				creative_list: order_data_clone.creative_list
			};

			<Orders>::insert(&order_id, &order);

			for billboard_data in order_data.clone().target_inventory.iter() {
				let billboard = Billboard {
					spot_duration: billboard_data.spot_duration,
					spots_per_hour: billboard_data.spot_duration,
					total_spots: billboard_data.total_spots,
					imp_multiplier_per_day: billboard_data.imp_multiplier_per_day
				};
				<Billboards>::insert(&order_id, &billboard_data.id, &billboard);
			}

			// Create Event Topic name
			let mut topic_name = Vec::new();
			topic_name.extend_from_slice(b"mw-reconciliation");
			let topic = T::Hashing::hash(&topic_name[..]);

			let event = <T as Trait>::Event::from(RawEvent::OrderSet(sender, order_id, order_data));
			frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());
		}

		#[weight = (T::WeightInfo::set_session_data(), Pays::No)]
		fn set_session_data(origin, session_data: SessionData) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Create Event Topic name
			let mut topic_name = Vec::new();
			topic_name.extend_from_slice(b"mw-reconciliation");
			let topic = T::Hashing::hash(&topic_name[..]);

			let order_exists = <Orders>::contains_key(&session_data.order_id);

			if order_exists {
				let billboard_exist = <Billboards>::contains_key(&session_data.order_id, &session_data.billboard_id);

				if billboard_exist {
					let order = <Orders>::get(&session_data.order_id);
					let creative_exists = order.creative_list.contains(&session_data.creative_id);

					if creative_exists {
						if session_data.timestamp >= order.start_date && session_data.timestamp <= order.end_date {
							let billboard = <Billboards>::get(&session_data.order_id, &session_data.billboard_id);

							if session_data.duration >= billboard.spot_duration {
								Self::update_verified_spots(session_data.clone(), billboard.clone());

								let event = <T as Trait>::Event::from(RawEvent::SessionDataProcessed(sender, session_data, false, b"".to_vec()));
								frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

								Ok(())
							} else {
								let event = <T as Trait>::Event::from(RawEvent::SessionDataProcessed(sender, session_data, true, b"Duration is lower than expected".to_vec()));
								frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

								Ok(())
							}
						} else {
							let event = <T as Trait>::Event::from(RawEvent::SessionDataProcessed(sender, session_data, true, b"Timestamp out of Order period range".to_vec()));
							frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

							Ok(())
						}
					} else {
						let event = <T as Trait>::Event::from(RawEvent::SessionDataProcessed(sender, session_data, true, b"Creative ID does not exist".to_vec()));
						frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

						Ok(())
					}
				} else {
					let event = <T as Trait>::Event::from(RawEvent::SessionDataProcessed(sender, session_data, true, b"Billboard ID does not exist".to_vec()));
					frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

					Ok(())
				}

				// // let naive_datetime = NaiveDateTime::from_timestamp(session_data.timestamp, 0);
				// // let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);

				// // let year = datetime.year().to_ne_bytes();
				// // let year = 1234_i32.to_str().into_bytes();
				// let year = 1234_i32;
				// let a: u32 = year as u32;
				// let mut data = String::<U32>::from("");
				// let year_good = write!(data,"{}", a);

				// let ola = match year_good {
				// 	Ok(ola) => ola,
				// 	// Err(_) => return Result<(), DispatchError::BadOrigin>,
				// 	Err(_) => return Err(Error::<T>::InvalidTimestamp)?,
				// };
				// // let b: String = a.to_string();
				// 		// let year = datetime.year().to_ne_bytes();
				// // let a = &year;
				// // let month = datetime.month();
				// // let day = datetime.day();

				// // // let dt = Utc.timestamp(session_data.timestamp, 0).to_string();
				// // // let dt = Utc.timestamp(1613644930, 0).to_rfc2822();
				// // // let dt = Utc::now().to_string();
				// // let pis = Utc.timestamp(1613644930, 0);
			 	// // let lol = TimeZone::offset_from_utc_datetime(utc: &pis);

				// // let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), lol);
				// // let caca = dt.to_rfc2822();

				// let date_time_after_a_billion_seconds = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
				// let a = date_time_after_a_billion_seconds.format("%a %b %e %T %Y").to_string();
			} else {
				let event = <T as Trait>::Event::from(RawEvent::SessionDataProcessed(sender, session_data, true, b"Order ID does not exist".to_vec()));
				frame_system::Module::<T>::deposit_event_indexed(&[topic], event.into());

				Ok(())
			}
		}
	}
}

impl<T: Trait> Module<T> {
	fn update_verified_spots(session_data: SessionData, billboard: Billboard) {
		let order_date_exists = <OrdersDate>::contains_key(&session_data.order_id, &session_data.date);
		let order_date: OrderDate;
		// let billboard = <Billboards>::get(&session_data.order_id, &session_data.billboard_id);

		if !order_date_exists {
			order_date = Self::create_order_date(session_data.clone());
		} else {
			order_date = <OrdersDate>::get(&session_data.order_id, &session_data.date);
		}

		let mut verified_spot = <VerifiedSpots>::get(&order_date, &session_data.billboard_id);
		verified_spot.verified_audience += billboard.imp_multiplier_per_day;

		<VerifiedSpots>::insert(&order_date, &session_data.billboard_id, verified_spot);
	}

	fn create_order_date(session_data: SessionData) -> OrderDate {
		let mut order_date = session_data.clone().order_id;
		let date = session_data.clone().date;

		order_date.extend(b"-".to_vec());
		order_date.extend(date);

		<OrdersDate>::insert(&session_data.order_id, &session_data.date, order_date.clone());
		order_date
	}
}
