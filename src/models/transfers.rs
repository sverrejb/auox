use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransferDTO {
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub to_account: String,
    pub from_account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferToCreditCardDTO {
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    pub from_account: String,
    pub credit_card_account_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResponse {
    #[serde(default)]
    pub errors: Vec<ErrorDTO>,
    pub payment_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDTO {
    pub code: String,
    pub message: String,
    pub trace_id: String,
    pub http_code: i32,
    pub resource: Option<String>,
    pub localized_message: Option<LocalizedMessage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedMessage {
    pub locale: Option<String>,
    pub message: Option<String>,
}
