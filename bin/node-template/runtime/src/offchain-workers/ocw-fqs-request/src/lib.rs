#![cfg_attr(not(feature = "std"), no_std)]

use core::{convert::TryInto, fmt};

// For better debugging (printout) support
use frame_support::{
	storage, debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
};

use parity_scale_codec::{Decode, Encode};

use frame_system::{
	self as system, ensure_signed,
	offchain::{Signer, CreateSignedTransaction, AppCrypto, SendSignedTransaction},
};

use sp_runtime::{
	RuntimeDebug,
	offchain as rt_offchain,
	offchain::{
		storage::StorageValueRef,
		storage_lock::{StorageLock, BlockAndTime},
	},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity,
		ValidTransaction,
	},
};

use sp_std::{
	prelude::*, str,
	// collections::vec_deque::VecDeque,
};

use sp_io::{
	storage::root as storage_root, storage::changes_root as storage_changes_root,
	hashing::{blake2_256, blake2_128, twox_128}, trie,
};

// We use `alt_serde`, and Xanewok-modified `serde_json` so that we can compile the program
//   with serde(features `std`) and alt_serde(features `no_std`).
use alt_serde::{Deserialize, Deserializer};

use sp_core::{crypto::KeyTypeId};

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When an offchain worker is signing transactions it's going to request keys from type
/// `KeyTypeId` via the keystore to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"fqs!");

// We are fetching information from the github public API about organization`substrate-developer-hub`.
pub const HTTP_REMOTE_REQUEST: &str = "https://api.github.com/orgs/substrate-developer-hub";
pub const HTTP_HEADER_USER_AGENT: &str = "jimmychu0807";

pub const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds
pub const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD + 1000; // in milli-seconds
pub const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

const MODULE: &[u8] = b"System";
const EVENT_TOPIC_STORAGE: &[u8] = b"EventTopics";
const EVENT_STORAGE: &[u8] = b"ExtrinsicData";
const EVENT_TOPIC_NAME: &[u8] = b"provenance-ledger-group";

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrapper.
/// We can utilize the supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// them with the pallet-specific identifier.
pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct OcwFQSAuthId;
	// implemented for ocw-runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwFQSAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	//// implemented for mock runtime in test
	// impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
	// 	for TestAuthId
	// {
	// 	type RuntimeAppPublic = Public;
	// 	type GenericSignature = sp_core::sr25519::Signature;
	// 	type GenericPublic = sp_core::sr25519::Public;
	// }
}

// Specifying serde path as `alt_serde`
// ref: https://serde.rs/container-attrs.html#crate
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Clone, Decode, Default, PartialEq)]
struct GithubInfo {
	// Specify our own deserializing function to convert JSON string to vector of bytes
	#[serde(deserialize_with = "de_string_to_bytes")]
	login: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	blog: Vec<u8>,
	public_repos: u32,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

impl fmt::Debug for GithubInfo {
	// `fmt` converts the vector of bytes inside the struct back to string for
	//   more friendly display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{{ login: {}, blog: {}, public_repos: {} }}",
			str::from_utf8(&self.login).map_err(|_| fmt::Error)?,
			str::from_utf8(&self.blog).map_err(|_| fmt::Error)?,
			&self.public_repos
		)
	}
}

/// This is the pallet's configuration trait
pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> {
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	/// Events generated by the module.
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// Event generated when a new number is accepted to contribute to the average.
		NewNumber(Option<AccountId>, u32),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		// // Error returned when not sure which ocw function to executed
		// UnknownOffchainMux,

		// // Error returned when making signed transactions in off-chain worker
		NoLocalAcctForSigning,
		OffchainSignedTxError,

		// // Error returned when making unsigned transactions in off-chain worker
		// OffchainUnsignedTxError,

		// // Error returned when making unsigned transactions with signed payloads in off-chain worker
		// OffchainUnsignedTxSignedPayloadError,

		// Error returned when fetching github info
		HttpFetchingError,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		// #[weight = 0]
		// pub fn submit_fqs_request_result(origin, public_repos: u32) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;
		// 	debug::info!("submit_number_signed: ({:?})", who);

		// 	Self::deposit_event(RawEvent::NewNumber(Some(who), public_repos));
		// 	Ok(())
		// }

		// fn offchain_worker(block: T::BlockNumber) {
		// 	// storage::hashed::get_or(&blake2_256, &who.to_keyed_vec(NONCE_OF), 0)
		// 	let module_key = twox_128(MODULE);
		// 	// let event_topic_storage_key = twox_128(EVENT_TOPIC_STORAGE);
		// 	let event_topic_storage_key = twox_128(EVENT_STORAGE);

		// 	// let event_topic_name = blake2_256(EVENT_TOPIC_NAME);
		// 	// let event_topic_hash = blake2_128(&event_topic_name);

		// 	// let provenance_ledger_events_key = &[&module_key[..], &event_topic_storage_key[..], &event_topic_hash[..], &event_topic_name[..]].concat();
		// 	let provenance_ledger_events_key = &[&module_key[..], &event_topic_storage_key[..]].concat();

		// 	// debug::info!("===================================== AAA =========================: {:?}", <system::Store<BlockHash>>::get());
		// 	debug::info!("===================================== KEY =========================: {:?}", *provenance_ledger_events_key);

		// 	debug::info!("===================================== VALUES =========================: {:?}", storage::hashed::get_or(&blake2_256, &provenance_ledger_events_key, 0));

		// 	if sp_io::offchain::is_validator() == true {
		// 		// Here I have to check for
		// 		Self::fqs_request_signed_tx(block);
		// 	}
		// }
	}
}

// impl<T: Trait> Module<T> {
// 	fn fqs_request_signed_tx(block_number: T::BlockNumber) -> Result<(), Error<T>> {
// 		// We retrieve a signer and check if it is valid.
// 		//   Since this pallet only has one key in the keystore. We use `any_account()1 to
// 		//   retrieve it. If there are multiple keys and we want to pinpoint it, `with_filter()` can be chained,
// 		//   ref: https://substrate.dev/rustdocs/v2.0.0/frame_system/offchain/struct.Signer.html
// 		let signer = Signer::<T, T::AuthorityId>::all_accounts();
// 		if !signer.can_sign() {
// 			debug::error!("No local account available");
// 			return Err(<Error<T>>::NoLocalAcctForSigning)
// 		}


// 		match Self::fetch_and_parse() {
// 			Ok(gh_info) => {

// 				// `result` is in the type of `Option<(Account<T>, Result<(), ()>)>`. It is:
// 				//   - `None`: no account is available for sending transaction
// 				//   - `Some((account, Ok(())))`: transaction is successfully sent
// 				//   - `Some((account, Err(())))`: error occured when sending the transaction
// 				let result = signer.send_signed_transaction(|_account|
// 					// This is the on-chain function
// 					Call::submit_fqs_request_result(gh_info.public_repos)
// 				);

// 				for (acc, res) in &result {
// 					match res {
// 						Err(e) => {
// 							debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
// 							return Err(<Error<T>>::OffchainSignedTxError);
// 						},
// 						Ok(()) => debug::info!("Success signing the tx")
// 					}
// 				}

// 				Ok(())

// 			}
// 			Err(err) => { return Err(err) }
// 		}

// 	}

// 	/// Fetch from remote and deserialize the JSON to a struct
// 	fn fetch_and_parse() -> Result<GithubInfo, Error<T>> {
// 		let resp_bytes = Self::fetch_from_remote().map_err(|e| {
// 			debug::error!("fetch_from_remote error: {:?}", e);
// 			<Error<T>>::HttpFetchingError
// 		})?;

// 		let resp_str = str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
// 		// Print out our fetched JSON string
// 		debug::info!("{}", resp_str);

// 		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
// 		let gh_info: GithubInfo =
// 			serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError)?;
// 		Ok(gh_info)
// 	}

// 	/// This function uses the `offchain::http` API to query the remote github information,
// 	///   and returns the JSON response as vector of bytes.
// 	fn fetch_from_remote() -> Result<Vec<u8>, Error<T>> {
// 		debug::info!("sending request to: {}", HTTP_REMOTE_REQUEST);

// 		// Initiate an external HTTP GET request. This is using high-level wrappers from `sp_runtime`.
// 		let request = rt_offchain::http::Request::get(HTTP_REMOTE_REQUEST);

// 		// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
// 		let timeout = sp_io::offchain::timestamp()
// 			.add(rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD));

// 		// For github API request, we also need to specify `user-agent` in http request header.
// 		//   See: https://developer.github.com/v3/#user-agent-required
// 		let pending = request
// 			.add_header("User-Agent", HTTP_HEADER_USER_AGENT)
// 			.deadline(timeout) // Setting the timeout time
// 			.send() // Sending the request out by the host
// 			.map_err(|_| <Error<T>>::HttpFetchingError)?;

// 		// By default, the http request is async from the runtime perspective. So we are asking the
// 		//   runtime to wait here.
// 		// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
// 		//   ref: https://substrate.dev/rustdocs/v2.0.0/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
// 		let response = pending
// 			.try_wait(timeout)
// 			.map_err(|_| <Error<T>>::HttpFetchingError)?
// 			.map_err(|_| <Error<T>>::HttpFetchingError)?;

// 		if response.code != 200 {
// 			debug::error!("Unexpected http request status code: {}", response.code);
// 			return Err(<Error<T>>::HttpFetchingError);
// 		}

// 		// Next we fully read the response body and collect it to a vector of bytes.
// 		Ok(response.body().collect::<Vec<u8>>())
// 	}


// }
