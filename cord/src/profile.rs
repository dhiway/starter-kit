use keystore::keystore::CordKeystoreSigner;

use subxt_rpcs::RpcClient;
// use subxt::ext::scale_value::Value;
use subxt::ext::scale_value::{value, Value, ValueDef};
// use subxt::{dynamic::Value, ext::scale_value::ValueDef};
use subxt::config::PolkadotConfig; // or your chain's config
use subxt::client::OnlineClient;
use subxt::ext::scale_value::At;
use std::sync::Arc;
use sp_core::crypto::Ss58AddressFormat;
use sp_core::ed25519;
use sp_core::crypto::Ss58Codec;

pub async fn create_profile(
    cord_client: Arc<OnlineClient<PolkadotConfig>>, 
    signer: CordKeystoreSigner
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // get the OnlineClient<PolkadotConfig>
    let client: &OnlineClient<PolkadotConfig> = &*cord_client;

    let data: Value = value! { () };
    let payload = subxt::tx::dynamic("Profile", "set_profile", vec![data]);

    let events = client
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer)
        .await
        .map_err(|e| format!("Failed to sign and submit transaction: {e}"))?
        .wait_for_finalized_success()
        .await
        .map_err(|e| format!("Transaction was not finalized successfully: {e}"))?;
    
    for ev in events.iter() {
        let ev = ev?;
        if ev.pallet_name() == "Profile" && ev.variant_name() == "ProfileSet" {
            let fields = ev.field_values()?;
            let mut who = None;
            let mut identifier = None;

            for label in ["who", "identifier"] {
                if let Some(value) = fields.at(label) {
                    if let Some(bytes) = try_extract_bytes(value) {
                        match label {
                            "who" => {
                                who = Some(format_account_id_as_ss58(&bytes));
                            },
                            "identifier" => {
                                identifier = Some(
                                    String::from_utf8(bytes.clone())
                                        .unwrap_or_else(|_| format!("0x{}", hex::encode(bytes))),
                                );
                            },
                            _ => {},
                        }
                    }
                }
            }

            return Ok((who.unwrap_or_default(), identifier.unwrap_or_default()));
        }
    }

    Err("ProfileSet event not found".into())
}

// HELPERS
pub fn try_extract_bytes(value: &Value<u32>) -> Option<Vec<u8>> {
	fn collect_primitives(value: &Value<u32>, out: &mut Vec<u8>) {
		match &value.value {
			ValueDef::Primitive(p) => {
				if let Some(b) = p.as_u128() {
					out.push(b as u8);
				}
			},
			ValueDef::Composite(comp) => {
				for v in comp.values() {
					collect_primitives(v, out);
				}
			},
			_ => {},
		}
	}

	let mut out = vec![];
	collect_primitives(value, &mut out);

	if !out.is_empty() {
		Some(out)
	} else {
		None
	}
}

pub fn format_account_id_as_ss58(account_bytes: &[u8]) -> String {
	let format = Ss58AddressFormat::custom(29);
	match account_bytes.try_into() {
		Ok(pub_key_bytes) => {
			let pub_key: ed25519::Public = pub_key_bytes;
			pub_key.to_ss58check_with_version(format)
		},
		Err(_) => format!("0x{}", hex::encode(account_bytes)),
	}
}