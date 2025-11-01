use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub accounts: Vec<Account>,
    pub errors: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub key: String,
    pub account_number: String,
    pub iban: String,
    pub name: String,
    pub description: String,
    pub balance: f64,
    pub available_balance: f64,
    pub currency_code: String,
    pub owner: Owner,
    pub product_type: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub product_id: String,
    pub description_code: String,
    pub disposal_role: bool,
    pub account_properties: AccountProperties,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub age: i64,
    pub customer_key: String,
    pub ssn_key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountProperties {
    pub is_transfer_from_enabled: bool,
    pub is_transfer_to_enabled: bool,
    pub is_payment_from_enabled: bool,
    pub is_allowed_in_avtale_giro: bool,
    pub has_access: bool,
    pub is_balance_preferred: bool,
    pub is_flexi_loan: bool,
    pub is_codebitor_loan: bool,
    pub is_security_balance: bool,
    pub is_aksjesparekonto: bool,
    pub is_savings_account: bool,
    pub is_bonus_account: bool,
    pub user_has_right_of_disposal: bool,
    pub user_has_right_of_access: bool,
    pub is_owned: bool,
    pub is_withdrawals_allowed: bool,
    pub is_blocked: bool,
    pub is_hidden: bool,
    pub is_balance_updated_immediately_on_transfer_to: bool,
    pub is_default_payment_account: bool,
}
