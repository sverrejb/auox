use log::debug;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::sync::mpsc;
use tiny_http::{Response, Server};
use url::form_urlencoded;
use urlencoding::encode;

use crate::api;
use crate::fileio::{read_access_token_file, save_token_data_file};
use crate::models::TokenData;

pub fn auth(client_id: String, client_secret: String, financial_institution: String) {
    if let Some(token_data) = read_access_token_file() {
        if is_token_valid() {
            return;
        }

        debug!("Access token not valid, attempting to refresh...");

        if let Ok(new_token_data) =
            refresh_access_token(&client_id, &client_secret, &token_data.refresh_token)
        {
            save_token_data_file(&new_token_data);
            debug!("Token refreshed successfully");
            return;
        }
    }

    debug!("Token refresh failed, starting full OAuth flow...");
    let code = get_code(&client_id, &financial_institution);
    if let Ok(token_data) = get_access_token(&code, &client_id, &client_secret) {
        save_token_data_file(&token_data);
        debug!("Access token obtained and saved successfully");
    } else {
        panic!("Failed to obtain access token from OAuth flow");
    }
}

fn get_code(client_id: &str, financial_institution: &str) -> String {
    let port = 8321;
    let redirect_uri = format!("http://localhost:{port}");

    let server = Server::http(format!("127.0.0.1:{port}")).unwrap();

    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let query = request.url().split('?').nth(1).unwrap_or("");
            let params: HashMap<_, _> = form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect();

            if let Some(code) = params.get("code").cloned() {
                let response =
                    Response::from_string("✅ Authentication complete! You can close this tab.");
                request.respond(response).unwrap();

                tx.send(code).unwrap();
                break; // exit server loop
            }
        }
    });

    let auth_url = format!(
        "https://api.sparebank1.no/oauth/authorize?client_id={}&state=123&redirect_uri={}&finInst={}&response_type=code",
        client_id,
        encode(&redirect_uri),
        financial_institution
    );
    open::that(&auth_url).unwrap();

    println!("Waiting for OAuth callback on {redirect_uri}...");

    let code = rx.recv().unwrap();
    println!("Code: {}", code);
    code
}

fn get_access_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenData, Box<dyn std::error::Error>> {
    let client = Client::new();
    let redirect_uri = "http://localhost:8321";

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];

    let response = client
        .post("https://api.sparebank1.no/oauth/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().unwrap_or_default();
        return Err(format!("Token exchange failed with status {}: {}", status, error_body).into());
    }

    let token_data: TokenData = response.json()?;

    Ok(token_data)
}

fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenData, Box<dyn std::error::Error>> {
    let client = Client::new();

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let response = client
        .post("https://api.sparebank1.no/oauth/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()?;

    if !response.status().is_success() {
        return Err(format!("Token refresh failed with status: {}", response.status()).into());
    }

    let token_data: TokenData = response.json()?;

    Ok(token_data)
}

fn is_token_valid() -> bool {
    let response = api::hello_world();

    match response {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}
