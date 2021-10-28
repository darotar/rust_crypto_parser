use clap::{App, Arg};
use dotenv::{dotenv, var};
use google_sheets4::{api::ValueRange, Error, Sheets};
use hyper_rustls::HttpsConnector;
use log::{debug, error, info};
use log4rs;
use reqwest::Client;
use std::collections::HashMap;

mod cmc;
mod eod;
mod error;

use cmc::CMCResponse;
use eod::EODResponse;
use error::AppError;

use yup_oauth2::{read_service_account_key, ServiceAccountAuthenticator};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref SHEET_ID: &'static str = "1JuF7MpFIkZSixwnmuvgH5KN5iPzcH9Xd05hTL0glya0";
    static ref SECRET_PATH: &'static str = "secret.json";
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let matches = App::new("CurrencyTracker")
        .version("1.0")
        .author("Anton P. <fabledva@mail.ru>")
        .about("Learn Rust in one go")
        .arg(
            Arg::with_name("currency_list")
                .short("c")
                .long("currencies")
                .help("Pass the list of currencies you want to query")
                .min_values(1)
                .required(true),
        )
        .arg(
            Arg::with_name("etfs")
                .long("etfs")
                .help("Pass the ETF symbols to fetch prices for")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let currency_list = matches
        .value_of("currency_list")
        .expect("No currencies were being passed");

    let etfs = matches.value_of("etfs").expect("No ETF symbol passed");

    debug!("Querying the following currencies: {:?}", currency_list);

    let cmc_pro_api_key = var("CMC_PRO_API_KEY").expect("CMC key not set");
    let eod_token = var("EOD_TOKEN").expect("EOD token not set");

    if cmc_pro_api_key.is_empty() {
        error!("Empty CMC API KEY provided! Please set one via .env file");
        return Err(AppError::NoAPIKey);
    }

    let mut params = HashMap::new();

    params.insert("symbol", currency_list.to_string());

    let client = Client::new();
    let resp = client
        .get("https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest")
        .header("X-CMC_PRO_API_KEY", cmc_pro_api_key)
        .query(&params)
        .send()
        .await?;

    let currencies = resp.json::<CMCResponse>().await?;

    let etf = client
        .get(format!(
            "https://eodhistoricaldata.com/api/real-time/{}?api_token={}&fmt=json",
            etfs, eod_token
        ))
        .send()
        .await?;

    let amundi_etf = etf.json::<EODResponse>().await?;

    debug!("Fetched ETF: {}", amundi_etf.close);

    let coins = ValueRange {
        major_dimension: Some("COLUMNS".to_string()),
        range: Some(format!("{}!{}2:{}4", "Crypto", "C", "C").to_owned()),
        values: Some(vec![vec![
            currencies
                .data
                .get(&"BTC".to_owned())
                .unwrap()
                .quote
                .0
                .get("USD")
                .unwrap()
                .price
                .to_string(),
            currencies
                .data
                .get(&"ETH".to_owned())
                .unwrap()
                .quote
                .0
                .get("USD")
                .unwrap()
                .price
                .to_string(),
            currencies
                .data
                .get(&"DOGE".to_owned())
                .unwrap()
                .quote
                .0
                .get("USD")
                .unwrap()
                .price
                .to_string(),
        ]]),
    };

    let etfs = ValueRange {
        major_dimension: Some("COLUMNS".to_string()),
        range: Some(format!("{}!{}2:{}2", "ETFs", "C", "C").to_owned()),
        values: Some(vec![vec![amundi_etf.close.to_string()]]),
    };

    update_google_sheet(&SECRET_PATH, coins).await;
    update_google_sheet(&SECRET_PATH, etfs).await;

    Ok(())
}

async fn update_google_sheet(secret_path: &str, values: ValueRange) {
    let authenticator =
        ServiceAccountAuthenticator::builder(read_service_account_key(secret_path).await.unwrap())
            .build()
            .await
            .expect("failed to create authenticator");

    let hub = Sheets::new(
        hyper::Client::builder().build(HttpsConnector::with_native_roots()),
        authenticator,
    );

    let range = values.clone().range.unwrap();

    let result = hub
        .spreadsheets()
        .values_update(values.clone(), &SHEET_ID, &values.range.unwrap())
        .value_input_option("USER_ENTERED")
        .doit()
        .await;

    match result {
        Err(e) => match e {
            Error::HttpError(_)
            | Error::Io(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => eprintln!("{}", e),
        },
        Ok((_, _)) => info!("{} Updated range: {}", chrono::offset::Utc::now(), range),
    }
}
