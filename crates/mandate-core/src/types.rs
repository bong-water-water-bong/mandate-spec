use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A signed, attested purchasing mandate. See the v0 spec for field semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mandate {
    pub version: String,
    pub id: String,
    pub buyer: Buyer,
    pub intent: Intent,
    pub budget: Budget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Preferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policies: Option<Policies>,
    pub agent: Agent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<Approval>,
    pub lifecycle: Lifecycle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buyer {
    pub wallet: String,
    pub sig: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub alg: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub brief: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub max_total: Money,
    pub settlement_rails: Vec<String>,
    #[serde(default = "default_true")]
    pub fees_inclusive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tip_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub amount: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_grant: Option<VaultGrant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultGrant {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policies {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_window_days_min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substitutions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_destinations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_sharing: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub runtime_id: String,
    pub attestation: Attestation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcek_chain: Option<String>,
    pub report: String,
    pub report_data_binding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_required_above: Option<Money>,
    pub approval_sig: Option<Signature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    pub created_at: String,
    pub expires_at: String,
    pub nonce: String,
    pub bound_to_merchant: Option<String>,
    pub state: String,
}

fn default_true() -> bool {
    true
}
