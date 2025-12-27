use log::debug;
use reqwest::{
    Error,
    blocking::{Client, Response},
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue},
};
use tui_input::Input;

use crate::{
    AppState, View, fileio::read_access_token_file, models::{AccountData, CreateTransferDTO, TransactionResponse, TransferResponse, TransferToCreditCardDTO}
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

pub fn perform_transfer(app: &mut AppState) {
    // Get amount and validate
    let amount = app.amount_input.value().trim();
    if amount.is_empty() {
        debug!("Amount is empty, not performing transfer");
        return;
    }

    let from_account = match app.from_account {
        Some(idx) => &app.accounts[idx],
        None => {
            debug!("No from_account selected");
            return;
        }
    };

    let to_account = match app.to_account {
        Some(idx) => &app.accounts[idx],
        None => {
            debug!("No to_account selected");
            return;
        }
    };

    // Check if transferring to a credit card
    let is_credit_card = to_account.type_field == "CREDITCARD";

    let response = if is_credit_card {
        // Credit card transfer - does not support message field
        let credit_card_id = to_account
            .credit_card_account_id
            .as_ref()
            .expect("Credit card account missing credit_card_account_id");

        let transfer = TransferToCreditCardDTO {
            amount: amount.to_string(),
            due_date: None,
            from_account: from_account.account_number.clone(),
            credit_card_account_id: credit_card_id.clone(),
        };

        debug!("Performing credit card transfer: {:?}", transfer);
        create_credit_card_transfer(transfer)
    } else {
        // Regular account transfer
        let message = app.message_input.value().trim();
        let message = if message.is_empty() {
            None
        } else {
            Some(message.to_string())
        };

        let transfer = CreateTransferDTO {
            amount: amount.to_string(),
            due_date: None,
            message,
            to_account: to_account.account_number.clone(),
            from_account: from_account.account_number.clone(),
            currency_code: None,
        };

        debug!("Performing transfer: {:?}", transfer);
        create_transfer(transfer)
    };

    // Check for errors
    if response.errors.is_empty() {
        debug!("Transfer successful! Payment ID: {:?}", response.payment_id);

        // Reset state
        app.amount_input = Input::default();
        app.message_input = Input::default();
        app.from_account = None;
        app.to_account = None;

        // Navigate back to accounts view
        app.view_stack.clear();
        app.view_stack.push(View::Accounts);

        // Refresh accounts to show updated balances
        app.accounts = crate::get_accounts();
    } else {
        debug!("Transfer failed with {} error(s):", response.errors.len());
        for error in &response.errors {
            debug!(
                "  - [{}] {} (HTTP {}): {}",
                error.code, error.trace_id, error.http_code, error.message
            );
            if let Some(localized) = &error.localized_message {
                if let Some(msg) = &localized.message {
                    debug!("    Localized: {}", msg);
                }
            }
        }
    }
}


pub fn create_transfer(transfer: CreateTransferDTO) -> TransferResponse {
    let url = "https://api.sparebank1.no/personal/banking/transfer/debit";

    let transfer_response = client().post(url).json(&transfer).send();

    let data: TransferResponse = match transfer_response {
        Ok(response) => {
            let status = response.status();
            let text = response
                .text()
                .expect("Failed to get transfer response text");

            // Check if the HTTP request was successful
            if !status.is_success() {
                panic!(
                    "Transfer API returned HTTP {}: {}",
                    status,
                    text
                );
            }

            serde_json::from_str(&text).unwrap_or_else(|err| {
                panic!("Failed to parse transfer JSON: {}\nResponse was: {}", err, text)
            })
        }
        Err(err) => {
            panic!("Transfer request failed: {}", err)
        }
    };

    data
}

pub fn create_credit_card_transfer(transfer: TransferToCreditCardDTO) -> TransferResponse {
    let url = "https://api.sparebank1.no/personal/banking/transfer/creditcard/transferTo";

    let transfer_response = client().post(url).json(&transfer).send();

    let data: TransferResponse = match transfer_response {
        Ok(response) => {
            let status = response.status();
            let text = response
                .text()
                .expect("Failed to get credit card transfer response text");

            // Check if the HTTP request was successful
            if !status.is_success() {
                panic!(
                    "Credit card transfer API returned HTTP {}: {}",
                    status,
                    text
                );
            }

            serde_json::from_str(&text).unwrap_or_else(|err| {
                panic!("Failed to parse credit card transfer JSON: {}\nResponse was: {}", err, text)
            })
        }
        Err(err) => {
            panic!("Credit card transfer request failed: {}", err)
        }
    };

    data
}
