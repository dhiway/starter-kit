use iroh::SecretKey;
use sp_core::crypto::{KeyTypeId, SecretString, Pair};
use sp_core::{blake2_256, ed25519, sr25519};
use sc_keystore::{Keystore, LocalKeystore};
use std::{sync::Arc, fmt, path::PathBuf};
use anyhow::Result;
use anyhow::anyhow;
use tracing::info;

pub const CORD_KEY_TYPE: KeyTypeId = KeyTypeId(*b"cord");
// why is this not public? 
const STARTERKIT_KEY_TYPE: KeyTypeId = KeyTypeId(*b"skit");

pub trait SecretKeyExt {
    fn from_seed(seed: [u8; 32]) -> Self;
}

impl SecretKeyExt for SecretKey {
    fn from_seed(seed: [u8; 32]) -> Self {
        SecretKey::from_bytes(&seed)
    }
}

pub struct StarterkitKeystore {
    keystore: Arc<LocalKeystore>,
}

impl fmt::Debug for StarterkitKeystore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyStore")
            .field("keystore", &"<hidden>")
            .finish()
    }
}

impl StarterkitKeystore {
    // initialize a new KeyStore(this is the bootstrap process)
    // WHEN TO CALL: when bootstraping the node
    pub fn new(keystore_path: &PathBuf, secret: Option<SecretString>) -> Result<Self> {
        if secret.is_none() {
            info!("⚠️  No secret provided. Setting up the keystore without encryption.");
        }

        let keystore = LocalKeystore::open(keystore_path, secret)
            .map_err(|e| anyhow::anyhow!("Failed to open keystore: {}", e))?;

        Ok(Self {
            keystore: Arc::new(keystore),
        })
    }

    // get the inner keystore from struct
    // WHEN TO CALL: when you need the inner keystore. eg. to sign tx
    pub fn inner(&self) -> Arc<LocalKeystore> {
        Arc::clone(&self.keystore)
    }

    // open a keystore with a given secret
    // WHEN TO CALL: when the node has already been bootstraped and we are restarting it
    //               after it had been stopped.
    pub fn open(keystore_path: &PathBuf, secret: Option<SecretString>) -> Result<Self> {
        if !keystore_path.exists() {
            return Err(anyhow!(
                "Keystore not found at {:?}. Please run the bootstrap process first.",
                keystore_path
            ));
        }

        // the only errors that can occur when opening a keystore are related to the secret:
        // -> if password is required, but none is given
        // -> if incorrect password is given
        // we need not cover more errors regarding file corruption or insufficient permissions.
        // NOTE: Even when passing a `password`, the keys on disk appear to look like normal secret
        //       uris. However, without having the correct password the secret uri will not generate
        //       the correct private key.
        let keystore = LocalKeystore::open(keystore_path, secret.clone())
            .map_err(|_| {
                if secret.is_none() {
                    anyhow!(
                       "❌ Keystore is password-protected. Please provide a password using --secret."
                    )
                } else {
                    anyhow!(
                       "❌ Failed to open keystore. Incorrect secret provided. Please verify and try again."
                    )
                }
            })?;

        Ok(Self {
            keystore: Arc::new(keystore),
        })
    }

    // right now this function does not look into any config or env files, it lonly looks at
    // the command line argument which shall be passed using --secret
    // TODO: pass the value of --secret from the command line arguments to here(also add
    //       env variable support).
    // WHEN TO CALL: before bootstraping the node, we would need to convert 'String' to 
    //               'SecretString' and pass it to the `new` function.
    pub fn keystore_access(secret_arg: Option<String>) -> Result<Option<SecretString>> {
        if let Some(secret) = secret_arg {
            Ok(Some(SecretString::new(secret)))
        } else {
            Ok(None)
        }
    }

    // After starting a new keystore, we would require to store the cord keypair in it.
    // We can not possibly store the private key of the CORD keypair in the keystore, so
    // we just store the public key and the secret URI (suri) of the keypair.
    // Question: How do we re-generate the private key using these 2 informations? 
    // WHEN TO CALL: after bootstraping the keystore, we would need to call this function
    //               to store the CORD keypair in the keystore.
    pub fn initialize_keystore(
        &mut self,
        suri: &str,
    ) -> Result<(sr25519::Public, ed25519::Public)> {
        // Create a sr25519 keypair from the provided secret URI
        let cord_pair = sr25519::Pair::from_string(suri.trim(), None)
            .map_err(|e| anyhow!("Failed to create sr25519 keypair for CORD: {e:?}"))?;
        
        // Get the public key from the keypair
        let cord_public = cord_pair.public();

        // Insert the keypair into the keystore
        // It will store something like this:
        //    b"cord" -> suri -> cord public key
        self.keystore
            .insert(CORD_KEY_TYPE, suri, cord_public.as_ref())
            .map_err(|e| anyhow!("Failed to insert CORD sr25519 keypair into keystore: {e:?}"))?;

        // Create an ed25519 keypair from the provided secret URI
        let starterkit_pair = ed25519::Pair::from_string(suri.trim(), None)
            .map_err(|e| anyhow!("Failed to create ed25519 keypair for StarterKit: {e:?}"))?;

        // Get the public key from the keypair
        let starterkit_public = starterkit_pair.public();

        // Insert the keypair into the keystore
        // It will store something like this:
        //    b"starterkit" -> suri -> starterkit public key
        self.keystore
            .insert(STARTERKIT_KEY_TYPE, suri, starterkit_public.as_ref())
            .map_err(|e| anyhow!("Failed to insert STARTERKIT ed25519 keypair into keystore: {e:?}"))?;

        Ok((cord_public, starterkit_public))
    }

    // This function returns the public key of the CORD keypair stored in the keystore.
    // It will return the first public key found for CORD.
    // Question: what if we add more public keys for CORD in the keystore?
    // WHEN TO CALL: when we need to retrieve the CORD public key from the keystore.
    //               This is useful when we need to sign a transaction with the CORD keypair.
    pub fn from_keystore(&self) -> Result<(sr25519::Public, ed25519::Public)> {
        let cord_public_keys = self.keystore.sr25519_public_keys(CORD_KEY_TYPE);

        // Assuming we want the first public key found for CORD
        let cord_public = cord_public_keys
            .get(0)
            .ok_or_else(|| anyhow!("No CORD public key found in the keystore."))?;

        let starterkit_public_keys = self.keystore.ed25519_public_keys(STARTERKIT_KEY_TYPE);

        // Assuming we want the first public key found for CORD
        let starterkit_public = starterkit_public_keys
            .get(0)
            .ok_or_else(|| anyhow!("No STARTERKIT public key found in the keystore."))?;

        Ok((cord_public.clone(), starterkit_public.clone()))
    }

    // get the public key of the CORD keypair from the keystore
    pub fn get_cord_public_key(&self) -> Result<sr25519::Public> {
        let cord_public_keys = self.keystore.sr25519_public_keys(CORD_KEY_TYPE);

        let cord_public = cord_public_keys
            .get(0)
            .ok_or_else(|| anyhow!("❌ CORD public key not found in keystore"))?;

        Ok(cord_public.clone())
    }

    // get the public key of the STARTERKIT keypair from the keystore
    pub fn get_starterkit_public_key(&self) -> Result<ed25519::Public> {
        let starterkit_public_keys = self.keystore.ed25519_public_keys(STARTERKIT_KEY_TYPE);

        let starterkit_public = starterkit_public_keys
            .get(0)
            .ok_or_else(|| anyhow!("❌ STARTERKIT public key not found in keystore"))?;

        Ok(starterkit_public.clone())
    }

    // NOTE: we will use this function to get the secret key for cyra to start the node. We were
    // earlier using a randomly generated secret key, but now we will make the secret key
    // DETERMINISTIC by using the public key of the starter kit.
    // We will use the private key to sign a payload and then use the public key and the 
    // signature(of signing the payload) to generate a seed for cyra.
    pub fn get_starter_kit_seed(&self, starter_kit_public: ed25519::Public) -> Result<SecretKey> {
        // NOTE: this is the message that will be signed using the private key.
        const PAYLOAD: &[u8] = b"starter_kit_key_derivation";

        // NOTE: this line is used to sign the payload with the CYRA keypair and generate a 
        //       ed25519 signature. The CYRA_KEY_TYPE and public key associated to cyra is 
        //       used to find the private key and then to sign the message.
        // NOTE: we are not yet sure how the private key is being generated from the public
        //       key to sign the message. But it should be noted that 'suri' is being stored in 
        //       the 'initialize_keystore' function, along with the public key, like this:
        //       b"starterkit" -> suri -> starterkit public key
        //       The 'ed25519::Pair::from_string' is using the 'suri' to generate the private
        //       key(suri can be used to fetch the public key). So maybe 'ed25519::ed25519_sign'
        //       function is using similar logic to generate the private key.
        //       IT SHOULD BE NOTED THAT THE PRIVATE KEY IS NOT STORED IN THE KEYSORE.
        let signature = self
            .keystore
            .ed25519_sign(STARTERKIT_KEY_TYPE, &starter_kit_public, PAYLOAD)
            .map_err(|e| {
                anyhow!("❌ Failed to sign payload with STARTERKIT keypair: {e:?}")
            })?
            .ok_or_else(|| {
                anyhow!("❌ Failed to sign payload with STARTERKIT keypair. No private key found.")
            })?;
        
        let mut combined_data = Vec::new();
        // Append the public key bytes to the combined data
        combined_data.extend_from_slice(starter_kit_public.as_ref());
        // Append the signature bytes to the combined data
        combined_data.extend_from_slice(&signature.as_ref());

        // Generate a seed from the combined data using blake3 hash function. 
        let seed = blake2_256(&combined_data);

        // Convert the seed into a SecretKey
        let secret_key = SecretKey::from_seed(seed);

        Ok(secret_key)
    }
}