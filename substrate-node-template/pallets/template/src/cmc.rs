//use reqwest::get;
use sp_std::prelude::*;
use alt_serde::Deserialize;
//use std::collections::{HashMap, HashSet};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use std::time::SystemTime;
use frame_support::debug;

const CMC_BASE_URL: &str = "https://pro-api.coinmarketcap.com";
const CM_BASE_URL: &str = "https://coinmetrics.io/api";

#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct CMCStatus {
    pub timestamp: Option<Vec<u8>>,
    pub error_code: u64,
    pub error_message: Option<Vec<u8>>,
    pub elapsed: u64,
    pub credit_count: u64,
}
#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct CMCQuote {
    pub price: f64,
    pub volume_24h: f64,
    pub percent_change_1h: f64,
    pub percent_change_24h: f64,
    pub percent_change_7d: f64,
    pub market_cap: f64,
    pub last_updated: Option<Vec<u8>>,
}
#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct CMCListing {
    pub id: u64,
    pub name: Vec<u8>,
    pub symbol: Vec<u8>,
    pub slug: Vec<u8>,
    pub cmc_rank: u64,
    pub num_market_pairs: u64,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub last_updated: Option<Vec<u8>>,
    pub date_added: Option<Vec<u8>>,
    pub quote: BTreeMap<Vec<u8>, CMCQuote>,
}
#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct CMCListingResponse {
    pub data: Vec<CMCListing>,
    pub status: CMCStatus,
}
impl CMCListingResponse {
    pub fn fill_usd(mut self) -> Self {
        let mut quote_map = BTreeMap::new();
        quote_map.insert(
            "USD".to_owned(),
            CMCQuote {
                price: 1.0,
                volume_24h: 0.0,
                percent_change_1h: 0.0,
                percent_change_24h: 0.0,
                percent_change_7d: 0.0,
                market_cap: 0.0,
                last_updated: None,
            },
        );
        self.data.push(CMCListing {
            id: 99999,
            name: b"USD".to_vec(),
            symbol: b"USD".to_vec(),
            slug: b"USD".to_vec(),
            cmc_rank: 99999,
            num_market_pairs: 1,
            circulating_supply: None,
            total_supply: None,
            max_supply: None,
            last_updated: None,
            date_added: None,
            quote: quote_map,
        });
        self
    }
}
#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct CMCHistoricalQuote {
    pub timestamp: Vec<u8>,
    pub quote: BTreeMap<Vec<u8>, CMCQuote>,
}
#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct CMCHistoricalQuotes {
    pub id: u64,
    pub name: Vec<u8>,
    pub symbol: Vec<u8>,
    pub quotes: Vec<CMCHistoricalQuote>,
}
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Debug)]
pub struct CMCHistoricalQuotesResponse {
    pub result: Vec<(u64, f64)>,
}
pub struct CMCClient {
    pub key: Vec<u8>,
}
impl CMCClient {
    pub fn new(key: Vec<u8>) -> Self {
        CMCClient { key: key }
    }
    pub fn latest_listings(&self, limit: u16) -> CMCListingResponse {
        let url: &str = &format!(
            "{}/v1/cryptocurrency/listings/latest?limit={}&CMC_PRO_API_KEY={}",
            CMC_BASE_URL, limit, self.key
        );
        let body: CMCListingResponse = match get(url) {
            Ok(mut data) => match data.json() {
                Ok(o) => o,
                Err(e) => {
                    debug::info!("{:?}", e);
                    panic!(e);
                }
            },
            Err(_) => CMCListingResponse {
                data: vec![],
                status: CMCStatus {
                    timestamp: Option::None,
                    error_code: 500,
                    error_message: Option::None,
                    elapsed: 1,
                    credit_count: 0,
                },
            },
        };
        body.fill_usd()
    }

    #[allow(unused)]
    pub fn historic_quotes(
        &self,
        symbol: &str,
        count: u64,
        _interval: &str,
    ) -> CMCHistoricalQuotesResponse {
        let current_epoch = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(), // println!("1970-01-01 00:00:00 UTC was {} seconds ago!", ),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        let seconds_in_day = 60 * 60 * 24;
        let extra_seconds = current_epoch % seconds_in_day;
        let beginning_of_today = current_epoch - extra_seconds;
        let beginning_of_period = beginning_of_today - (seconds_in_day * count);
        let l_symbol = &symbol.to_lowercase()[..];
        let url: &str = &format!(
            "{}/v1/get_asset_data_for_time_range/{}/marketcap(usd)/{}/{}",
            CM_BASE_URL, l_symbol, beginning_of_period, beginning_of_today
        );
        let body: CMCHistoricalQuotesResponse = match get(url) {
            Ok(mut data) => match data.json() {
                Ok(o) => o,
                Err(e) => {
                    debug::info!("{}", e);
                    debug::info!("{:?}", data);
                    panic!(e);
                }
            },
            Err(e) => panic!(e),
        };
        body
    }

    #[allow(unused)]
    pub fn supported_assets(&self) -> BTreeSet<Vec<u8>> {
        let url: &str = &format!("{}/v1/get_supported_assets", CM_BASE_URL);
        let body: BTreeSet<Vec<u8>> = match get(url) {
            Ok(mut data) => match data.json() {
                Ok(o) => o,
                Err(e) => {
                    debug::info!("{}", e);
                    debug::info!("{:?}", data);
                    panic!(e);
                }
            },
            Err(e) => panic!(e),
        };
        body.iter().map(|s| s.to_uppercase()).collect()
    }
}
