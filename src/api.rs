use reqwest::{
    Error,
    blocking::{Client, Response},
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue},
};

use crate::{
    fileio::read_access_token_file,
    models::{AccountData, TransactionResponse},
};

fn client() -> Client {
    let access_token = read_access_token_file().expect("Unable to read access token file!").access_token;

    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token))
            .expect("Invalid access token format"),
    );

    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.sparebank1.v1+json; charset=utf-8"),
    );

    Client::builder()
        .default_headers(headers)
        .build()
        .expect("Unable to create API client")
}

pub fn get_accounts() -> AccountData {

    let account_response = client()
        .get("https://api.sparebank1.no/personal/banking/accounts?includeCreditCardAccounts=true")
        .send();

    let data: AccountData = match account_response {
        Ok(response) => {
            let text = response
                .text()
                .expect("Failed to get account response text");
            serde_json::from_str(&text).expect("Failed to parse accounts JSON")
        }
        Err(err) => {
            panic!("Paniced: {}", err)
        }
    };

    data
}

pub fn get_transactions(account_key: &String) -> TransactionResponse {
    let url = format!(
        "https://api.sparebank1.no/personal/banking/transactions?accountKey={}",
        account_key
    );

    let transactions_respose = client().get(&url).send();

    let data: TransactionResponse = match transactions_respose {
        Ok(response) => {
            let text = response
                .text()
                .expect("Failed to get transactions response text");
            serde_json::from_str(&text).expect("Failed to parse transactions JSON")
        }
        Err(err) => {
            panic!("Shieeet: {}", err)
        }
    };
    data
}

pub fn hello_world() -> Result<Response, Error> {
    client()
        .get("https://api.sparebank1.no/common/helloworld")
        .send()
}
