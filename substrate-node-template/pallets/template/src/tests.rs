// Tests to be written here

use crate::{Error, mock::*, CoinCap};
use frame_support::{assert_ok, assert_noop};

#[test]
fn test_data_deserialize() {
	let json = "{\"data\":{\"id\":\"ethereum\",\"rank\":\"1\",\"symbol\":\"ETH\",\"name\":\"Ethereum\",\"supply\":\"111669845.374000\",\"maxSupply\":null,\"marketCapUsd\":\"26877027658.2790761\",\"volumeUsd24Hr\":\"1573167228.6739012\",\"priceUsd\":\"248.682948\",\"changePercent24Hr\":\"8.323286\",\"vwap24Hr\":\"239.0067810\"},\"timestamp\":1594897537505}";
	let coin = serde_json::from_str::<CoinCap>(json);
	if let Ok(coin_cap) = coin {
		assert_eq!(coin_cap.data.priceUsd, 248.682948);
	} else {
		panic!("serialize error");
	}
}

