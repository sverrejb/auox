use reqwest::{Error, blocking::Client, blocking::Response};

use crate::models::AccountData;

pub fn get_accounts(access_token: String) -> AccountData {
    let client = Client::new();

    let account_response = client
        .get("https://api.sparebank1.no/personal/banking/accounts?includeCreditCardAccounts=true")
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            "Accept",
            "application/vnd.sparebank1.v1+json; charset=utf-8",
        )
        .send();

    let data: AccountData = match account_response {
        Ok(response) => response.json().expect("Failed to parse JSON"),
        Err(err) => {
            panic!("Paniced: {}", err)
        }
    };

    data
}

pub fn hello_world(access_token: String) -> Result<Response, Error> {
    let client = Client::new();
    client
        .get("https://api.sparebank1.no/common/helloworld")
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            "Accept",
            "application/vnd.sparebank1.v1+json; charset=utf-8",
        )
        .send()
}
