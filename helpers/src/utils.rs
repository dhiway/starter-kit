use data_encoding::HEXLOWER;
use iroh_docs::AuthorId;
use sp_core::{
    crypto::{Ss58AddressFormat, Ss58Codec},
    ed25519,
    ed25519::Public,
};
use anyhow::{anyhow, Result};
use iroh_docs::store::{DownloadPolicy, FilterKind};
use serde_json;
use regex::Regex;
use axum::http::{HeaderMap, StatusCode};

/// Encode a byte array into a custom document identifier.
pub fn encode_doc_id(data: &[u8]) -> String {
    format!("d{}", HEXLOWER.encode(data))
}

/// Decode a custom-encoded string back into a fixed-size byte array, ignoring the prefix.
pub fn decode_doc_id(encoded: &str) -> Result<[u8; 32]> {
    let (prefix, data) = encoded.split_at(1);

    if prefix != "d" {
        return Err(anyhow::anyhow!("Invalid prefix"));
    }
    let decoded = HEXLOWER
        .decode(data.as_bytes())
        .map_err(|_| anyhow::anyhow!("Invalid hex string"))?;

    if decoded.len() != 32 {
        return Err(anyhow::anyhow!("Invalid length. Expected 32, got {}", decoded.len()));
    }

    let mut fixed_array = [0u8; 32];
    fixed_array.copy_from_slice(&decoded);

    Ok(fixed_array)
}

pub fn encode_key(key: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(key);
    encoded.push(0); // Append a null terminator
    encoded
}

pub fn decode_key(encoded_key: &[u8]) -> Vec<u8> {
    if encoded_key.ends_with(&[0]) {
        encoded_key[..encoded_key.len() - 1].to_vec()
    } else {
        encoded_key.to_vec()
    }
}

#[derive(Debug, Clone)]
pub struct SS58AuthorId(String);

impl SS58AuthorId {
    /// Convert from AuthorId to SS58 string.
    pub fn from_author_id(author_id: &AuthorId) -> Result<Self> {
        let public_key = author_id
            .into_public_key()
            .map_err(|e| anyhow!("Failed to retrieve public key: {e}"))?;

        // Set the SS58 address format (custom 29 used in Cyra)
        sp_core::crypto::set_default_ss58_version(Ss58AddressFormat::custom(29));

        let ss58_compatible_key = ed25519::Public::from_raw(*public_key.as_bytes());
        let ss58_string = ss58_compatible_key.to_ss58check();

        Ok(SS58AuthorId(ss58_string))
    }

    /// Convert an SS58 string back into AuthorId
    pub fn to_author_id(&self) -> Result<AuthorId> {
        let public_key = Public::from_string(&self.0)
            .map_err(|e| anyhow!("Invalid SS58 string: {e}"))?;
        Ok(AuthorId::from(public_key.0))
    }

    /// Direct decode helper from string to AuthorId
    pub fn decode(author_id: &str) -> Result<AuthorId> {
        let public_key = Public::from_string(author_id)
            .map_err(|e| anyhow!("Invalid author ID format: {e}"))?;
        Ok(AuthorId::from(public_key.0))
    }

    /// Get SS58 string
    pub fn as_ss58(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ApiDownloadPolicy(pub DownloadPolicy);

impl ApiDownloadPolicy {
    pub fn to_json(&self) -> serde_json::Value {
        let (policy_type, filters) = match &self.0 {
            DownloadPolicy::NothingExcept(filters) => ("nothing_except", filters),
            DownloadPolicy::EverythingExcept(filters) => ("everything_except", filters),
        };

        serde_json::json!({
            "policy": policy_type,
            "filters": filters.iter().map(|f| f.to_string()).collect::<Vec<_>>(),
        })
    }

    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        let policy_type = value
            .get("policy")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Expected 'policy' field to be a string"))?;

        let filters = value
            .get("filters")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Filters must be an array"))?
            .iter()
            .map(|f| {
                f.as_str()
                    .ok_or_else(|| anyhow!("Each filter must be a string"))
                    .and_then(|s| {
                        s.parse::<FilterKind>().map_err(|e| {
                            anyhow!("Invalid filter format: {}", e)
                        })
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let download_policy = match policy_type {
            "nothing_except" => DownloadPolicy::NothingExcept(filters),
            "everything_except" => DownloadPolicy::EverythingExcept(filters),
            _ => {
                return Err(anyhow!(
                    "Invalid policy type. Expected 'nothing_except' or 'everything_except'",
                ));
            }
        };

        Ok(ApiDownloadPolicy(download_policy))
    }
}

pub async fn validate_key(
    key: &str,
    check_reserved: bool,
) -> anyhow::Result<()> {
    let key_regex = Regex::new(r"^\S+$")
        .map_err(|e| anyhow::anyhow!("Failed to compile key validation regex: {}", e))?;

    if !key_regex.is_match(key) {
        return Err(anyhow::anyhow!("Invalid key format: Key must not contain spaces"));
    }

    if check_reserved && key.eq_ignore_ascii_case("schema") {
        return Err(anyhow::anyhow!("The key 'schema' is reserved for document operations"));
    }

    Ok(())
}

pub fn normalize_domain(input: &str) -> Option<String> {
    let no_scheme = input.trim().trim_start_matches("http://").trim_start_matches("https://");
    no_scheme.split('/').next().map(|s| s.to_lowercase())
}

// API handler function's header checks
pub fn get_author_id_from_headers(headers: &HeaderMap) -> Result<String, (StatusCode, String)> {
    headers
        .get("author-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing or invalid author-id header".to_string()))
}