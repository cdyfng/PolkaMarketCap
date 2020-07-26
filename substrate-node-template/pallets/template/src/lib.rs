#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{debug, decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{
	self as system, ensure_signed,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer, SubmitTransaction,
	},
};

use sp_runtime::{
	offchain as rt_offchain,
	offchain::storage::StorageValueRef,
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
		ValidTransaction,
	},
};

use sp_std::prelude::*;
use sp_std::str;

use codec::{Encode, Decode};
use alt_serde::{Deserialize, Deserializer};
mod cmc;

use crate::cmc::{CMCClient, CMCListing, CMCListingResponse};
use std::collections::HashMap;
//use std::time::{Duration, SystemTime};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// Ethereum api
pub const COINCAP_API_URL: &[u8] = b"https://api.coincap.io/v2/assets/ethereum";
pub const CRYPTOCOMPARE_API_URL: &[u8] = b"https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USD";
const CMC_KEY: &str = "845c42cb-56b3-483b-a32f-baea0ed5d874";

#[serde(crate = "alt_serde")]
#[derive(Debug, Deserialize)]
pub struct CoinCap {
	data: CoinCapData
}

#[serde(crate = "alt_serde")]
#[derive(Debug, Deserialize)]
pub struct CoinCapData {
	#[serde(deserialize_with = "de_string_to_f64")]
	priceUsd: f64
}

#[serde(crate = "alt_serde")]
#[derive(Debug, Deserialize)]
pub struct CryptoCompare {
	USD: f64
}

#[derive(Debug, Encode, Decode)]
pub struct EthereumPrices {
	prices: Vec<u64>,
}

fn de_string_to_f64<'der, D>(der: D) -> Result<f64, D::Error>
	where D: Deserializer<'der> {
	let s: &str = Deserialize::deserialize(der)?;
	Ok(s.parse::<f64>().unwrap())
}
/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(fn something): Option<u32>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Just a dummy event.
		/// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		/// To emit this event, we call the deposit function, from our runtime functions
		SomethingStored(u32, AccountId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,

		HttpFetchingError,

		ResponseFormatError,

		ResponseParseError,

		AlreadyFetched,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		#[weight = 10_000]
		pub fn save_number(origin, number: u32) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let who = ensure_signed(origin)?;

			/*******
			 * 学员们在这里追加逻辑
			 *******/

			Ok(())
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("Entering off-chain workers");

			/*******
			 * 学员们在这里追加逻辑
			 *******/
			let cmc = CMCClient::new(CMC_KEY.to_string());
	        //let now = SystemTime::now();
 	        let prices = cmc.latest_listings(30);

        	let price_map = Self::cmc_listings_as_map(&prices);
        	//debug::info!("{:?}", now);
        	for (key, value) in price_map {
            	debug::info!("{} / {}", key, value.quote.get("USD").unwrap().price);
            //println!("{} ", key);
            //map.remove(key);
        	}

			let price = Self::fetch_ethereum_price();
			let res = price.map(|p| Self::append_price(p));

			if let Err(e) = res {
				debug::error!("Error: {:?} at block number: {:?}", e, block_number);
			}
		}

	}
}

impl <T: Trait> Module<T> {

	fn cmc_listings_as_map<'a>(listing: &'a CMCListingResponse) -> HashMap<String, &'a CMCListing> {
   	 	let mut h_map = HashMap::new();
    	for l in &listing.data {
        	h_map.insert(l.symbol.to_owned(), l);
    	}
    	h_map
	}


	fn append_price(price: u64) -> Result<(), Error::<T>> {
		let ethereum_store = StorageValueRef::persistent(b"pallet-tempale::ethereum_price_store");
		let ethereum_lock = StorageValueRef::persistent(b"pallet-template::ethereum_price_lock");

		let res: Result<Result<bool, bool>, Error<T>> = ethereum_lock.mutate(|s: Option<Option<bool>>| {
			match s {
				// `s` can be one of the following:
				//   `None`: the lock has never been set. Treated as the lock is free
				//   `Some(None)`: unexpected case, treated it as AlreadyFetch
				//   `Some(Some(false))`: the lock is free
				//   `Some(Some(true))`: the lock is held
				None | Some(Some(false)) => Ok(true),
				_ => Err(Error::<T>::AlreadyFetched)
			}
		});

		if let Ok(Ok(true)) = res {
			let mut e_ps = EthereumPrices { prices: vec![] };
			if let Some(Some(prices)) = ethereum_store.get::<EthereumPrices>() {
				e_ps.prices = prices.prices;
			}
			e_ps.prices.push(price);
			debug::info!("current price list: {:?}", e_ps);

			ethereum_store.set(&e_ps);
			ethereum_lock.set(&false);
		}

		Ok(())

	}

	fn fetch_ethereum_price() -> Result<u64, Error::<T>> {
		let coin_cap_bytes = Self::fetch_data_from_remote(&COINCAP_API_URL)?;
		let crypto_compare_bytes = Self::fetch_data_from_remote(&CRYPTOCOMPARE_API_URL)?;

		let coin_cap_content = str::from_utf8(&coin_cap_bytes)
			.map_err(|_| Error::<T>::ResponseFormatError)?;
		let crypto_compare_content = str::from_utf8(&crypto_compare_bytes)
			.map_err(|_| Error::<T>::ResponseFormatError)?;

		let coin_cap = serde_json::from_str::<CoinCap>(coin_cap_content)
			.map_err(|_| Error::<T>::ResponseParseError)?;
		debug::info!("coincap object: {:?}", coin_cap);

		let crypto_compare = serde_json::from_str::<CryptoCompare>(crypto_compare_content)
			.map_err(|_| Error::<T>::ResponseParseError)?;
		debug::info!("cryptocompare object: {:?}", crypto_compare);

		let price = (coin_cap.data.priceUsd as u64 + crypto_compare.USD as u64) / 2;
		Ok(price)
	}

	fn fetch_data_from_remote(url_bytes: &[u8]) -> Result<Vec<u8>, Error<T>> {

		let remote_url = str::from_utf8(url_bytes)
			.map_err(|_| <Error<T>>::HttpFetchingError)?;
		let request = rt_offchain::http::Request::get(remote_url);

		let timeout = sp_io::offchain::timestamp().add(rt_offchain::Duration::from_millis(3000));

		let pending = request
			// .add_header("Content-Type", str::from_utf8(b"application/json;charset=utf-8")
			.deadline(timeout)
			.send()
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		let response = pending
			.try_wait(timeout)
			.map_err(|_| Error::<T>::HttpFetchingError)?
			.map_err(|_| Error::<T>::HttpFetchingError)?;

		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);

			return Err(<Error<T>>::HttpFetchingError);
		}

		Ok(response.body().collect::<Vec<u8>>())
	}
}
