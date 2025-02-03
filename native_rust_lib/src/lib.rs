#![allow(unused_imports)] // Just to avoid spurious warnings in this example.
// --- Dependencies ---
use jni::objects::{JByteArray, JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jbyteArray, jint, jlong, jobject, jstring};
use jni::JNIEnv;

// (1) Pull in android_logger, log, and ctor crates.
use android_logger::Config as AndroidLogConfig;
use log::Level;
use log::LevelFilter;
use ctor::ctor;

// Once-cell, arcs, sync, etc.
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use penumbra_asset::{asset, Value};
use penumbra_keys::keys::Diversifier;
use penumbra_keys::{
    keys::{Bip44Path, SeedPhrase, SpendKey, SpendKeyBytes},
    Address,
};
use penumbra_num::Amount;
use penumbra_proto::DomainType;
use penumbra_proto::{penumbra::core::keys::v1 as pb, Message};
use penumbra_shielded_pool::Rseed;
use std::str::FromStr;

use cosmwasm_std::{Binary, HexBinary};
use decaf377::{Fq, Fr};
use decaf377_fmd as fmd;
use decaf377_ka as ka;
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};


use rand::RngCore;
use serde::{Deserialize, Serialize};

pub mod note;
pub mod r1cs;


uniffi::setup_scaffolding!();



// (2) Initialize the Android logger once at startup (only on Android).
#[cfg(target_os = "android")]
#[ctor]
static INIT_LOGGER: () = {
    android_logger::init_once(
        AndroidLogConfig::default()
            // log levels at or above INFO will be printed
            .with_max_level(LevelFilter::Info)
            .with_tag("RustNative") // choose your tag
    );
};

// (3) A convenience macro for logging. Use it instead of println! or eprintln!.
#[macro_export]
macro_rules! rust_log {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

// Core FFI Types
#[derive(uniffi::Record, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressData {
    pub diversifier: Vec<u8>,
    pub transmission_key: Vec<u8>,
    pub clue_key: Vec<u8>,
}

#[derive(uniffi::Record, Serialize, Deserialize)]
pub struct KeyPair {
    pub spend_key: Vec<u8>,
    pub view_key: Vec<u8>,
}

#[derive(uniffi::Record, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub debtor_address: AddressData,
    pub creditor_address: AddressData,
    pub amount: u64,
    pub asset_id: u64,
    pub commitment: Vec<u8>,
}

#[derive(uniffi::Record, Serialize, Deserialize)]
pub struct SignedNote {
    pub note: Note,
    pub signature: Vec<u8>,
    pub verification_key: Vec<u8>,
}

#[derive(Serialize)]
pub struct CreateIntentAction {
    pub note_commitment: HexBinary,
    pub auth_sig: HexBinary,
    pub rk: HexBinary,
    pub zkp: HexBinary,
    pub ciphertexts: [Binary; 2],
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ProofError {
    #[error("Invalid seed phrase")]
    InvalidSeed,
    #[error("Invalid key")]
    InvalidKey,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Note creation failed: {0}")]
    NoteError(String),
    #[error("Intent creation failed: {0}")]
    IntentError(String),
}

static PROOF_MANAGER: Lazy<Mutex<Arc<ProofManager>>> =
    Lazy::new(|| Mutex::new(ProofManager::new().expect("Failed to initialize ProofManager")));

#[derive(uniffi::Object)]
pub struct ProofManager {
    spend_auth_randomizer: Fr,
}

#[uniffi::export]
impl ProofManager {
    #[uniffi::constructor]
    pub fn new() -> Result<Arc<Self>, ProofError> {
        Ok(Arc::new(Self {
            spend_auth_randomizer: Fr::rand(&mut rand::thread_rng()),
        }))
    }

    pub fn generate_keys(&self, seed_phrase: String) -> Result<KeyPair, ProofError> {
        let seed = SeedPhrase::from_str(&seed_phrase).map_err(|_| ProofError::InvalidSeed)?;

        let spend_key = SpendKey::from_seed_phrase_bip44(seed, &Bip44Path::new(0));
        let view_key = spend_key.full_viewing_key();

        Ok(KeyPair {
            spend_key: spend_key.to_bytes().0.to_vec(),
            view_key: view_key.nullifier_key().0.to_bytes().to_vec(),
        })
    }

    pub fn generate_address(
        &self,
        spend_key_bytes: Vec<u8>,
        index: u32,
    ) -> Result<AddressData, ProofError> {
        let spend_key_bytes: [u8; 32] = spend_key_bytes
            .try_into()
            .map_err(|_| ProofError::InvalidKey)?;
        let spend_key = SpendKey::from(SpendKeyBytes(spend_key_bytes));

        let fvk = spend_key.full_viewing_key();
        let ivk = fvk.incoming();
        let (address, _) = ivk.payment_address(index.into());

        Ok(AddressData {
            diversifier: address.diversifier().0.to_vec(),
            transmission_key: address.transmission_key().0.to_vec(),
            clue_key: address.clue_key().0.to_vec(),
        })
    }

    pub fn create_note(
        &self,
        debtor_address: AddressData,
        creditor_address: AddressData,
        amount: u64,
        asset_id: u64,
    ) -> Result<Note, ProofError> {
        let debtor_addr = Address::from_components(
            Diversifier(
                debtor_address
                    .diversifier
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
            ka::Public(
                debtor_address
                    .transmission_key
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
            fmd::ClueKey(
                debtor_address
                    .clue_key
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
        )
        .ok_or_else(|| ProofError::InvalidKey)?;

        let creditor_addr = Address::from_components(
            Diversifier(
                creditor_address
                    .diversifier
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
            ka::Public(
                creditor_address
                    .transmission_key
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
            fmd::ClueKey(
                creditor_address
                    .clue_key
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
        )
        .ok_or_else(|| ProofError::InvalidKey)?;

        let value = Value {
            amount: Amount::from(amount),
            asset_id: asset::Id(Fq::from(asset_id)),
        };

        let mut rng = rand::thread_rng();
        let mut rseed_bytes = [0u8; 32];
        rng.fill_bytes(&mut rseed_bytes);

        let note = note::Note::from_parts(debtor_addr, creditor_addr, value, Rseed(rseed_bytes))
            .map_err(|e| ProofError::NoteError(e.to_string()))?;

        let commitment = note.commit().0.to_bytes();

        Ok(Note {
            debtor_address,
            creditor_address,
            amount,
            asset_id,
            commitment: commitment.to_vec(),
        })
    }

    pub fn create_intent_action(
        &self,
        debtor_seed_phrase: Vec<u8>,
        rseed_randomness: Vec<u8>,
        debtor_index: u32,
        creditor_addr: String,
        amount: u64,
        asset_id: u64,
    ) -> Result<String, ProofError> {

      rust_log!("create_intent_action called");
      rust_log!("debtor_seed_phrase len = {}", debtor_seed_phrase.len());
      rust_log!("rseed_randomness len = {}", rseed_randomness.len());
      rust_log!("creditor_addr = {}", creditor_addr);

        let sk_debtor = {
            let seed_phrase = SeedPhrase::from_randomness(&debtor_seed_phrase);
            SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0))
        };
        rust_log!("Generated debtor spend key {:?}", sk_debtor);

        let debtor_addr = {
            let fvk = sk_debtor.full_viewing_key();
            let ivk = fvk.incoming();
            ivk.payment_address(debtor_index.into()).0
        };
        rust_log!("Successfully parsed debtor address {:?}", debtor_addr);

        let rsk_debtor = sk_debtor
            .spend_auth_key()
            .randomize(&self.spend_auth_randomizer);
        let rk_debtor: VerificationKey<SpendAuth> = rsk_debtor.into();

        let value = Value {
            amount: Amount::from(amount),
            asset_id: asset::Id(Fq::from(asset_id)),
        };

        // Convert to proto address first
        let proto_addr = pb::Address {
            inner: Vec::new(),
            alt_bech32m: creditor_addr,
        }.encode_to_vec();
        
        let creditor_address = Address::decode(proto_addr.as_ref())
            .map_err(|_| ProofError::InvalidKey)?;
        rust_log!("Successfully parsed creditor address {:?}", creditor_address);

        let note = note::Note::from_parts(
            debtor_addr.clone(),
            creditor_address,
            value,
            Rseed(
                rseed_randomness[..32]
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
        )
        .map_err(|e| ProofError::NoteError(e.to_string()))?;

        let note_commitment = HexBinary::from(<[u8; 32]>::from(note.commit()));
        let auth_sig = HexBinary::from(Vec::<u8>::from(
            rsk_debtor.sign(rand::thread_rng(), note_commitment.as_slice()),
        ));
        let rk = HexBinary::from(rk_debtor.to_bytes());

        let create_intent_action = CreateIntentAction {
            note_commitment,
            auth_sig,
            rk,
            zkp: Default::default(),
            ciphertexts: [
                Binary::new(
                    serde_json::to_vec(&note)
                        .map_err(|e| ProofError::IntentError(e.to_string()))?,
                ),
                Default::default(),
            ],
        };

        serde_json::to_string(&create_intent_action)
            .map_err(|e| ProofError::IntentError(e.to_string()))

          
    }

    pub fn sign_note(&self, seed_phrase: String, note: Note) -> Result<SignedNote, ProofError> {
        let seed = SeedPhrase::from_str(&seed_phrase).map_err(|_| ProofError::InvalidSeed)?;
        let spend_key = SpendKey::from_seed_phrase_bip44(seed, &Bip44Path::new(0));

        let rsk = spend_key
            .spend_auth_key()
            .randomize(&self.spend_auth_randomizer);
        let rk: VerificationKey<SpendAuth> = rsk.into();

        let signature = {
            let sig = rsk.sign(rand::thread_rng(), &note.commitment);
            Vec::<u8>::from(sig)
        };

        Ok(SignedNote {
            note,
            signature,
            verification_key: rk.to_bytes().to_vec(),
        })
    }

    pub fn verify_signature(
        &self,
        verification_key_bytes: Vec<u8>,
        commitment: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<bool, ProofError> {
        let rk = VerificationKey::<SpendAuth>::try_from(verification_key_bytes.as_slice())
            .map_err(|_| ProofError::InvalidKey)?;

        let sig =
            Signature::try_from(signature.as_slice()).map_err(|_| ProofError::InvalidSignature)?;

        Ok(rk.verify(&commitment, &sig).is_ok())
    }
}

#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_generateKeysNative<
    'local,
>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    seed_phrase: JString<'local>,
) -> jobject {
    let seed_phrase: String = env
        .get_string(&seed_phrase)
        .expect("Couldn't get java string!")
        .into();

    match PROOF_MANAGER.lock().unwrap().generate_keys(seed_phrase) {
        Ok(key_pair) => {
            let hash_map_class = env
                .find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            let hash_map = env
                .new_object(hash_map_class, "()V", &[])
                .expect("Failed to create HashMap");

            // Helper function to add byte array to map
            let mut put_bytes = |key: &str, bytes: &[u8]| {
                let j_key = env.new_string(key).expect("Failed to create string");
                let j_value = env
                    .byte_array_from_slice(bytes)
                    .expect("Failed to create byte array");

                env.call_method(
                    &hash_map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[
                        JValue::Object(&j_key.into()),
                        JValue::Object(&j_value.into()),
                    ],
                )
                .expect("Failed to call put");
            };

            put_bytes("spendKey", &key_pair.spend_key);
            put_bytes("viewKey", &key_pair.view_key);

            hash_map.into_raw()
        }
        Err(e) => {
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            let hash_map_class = env
                .find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create empty HashMap")
                .into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_generateAddressNative<
    'local,
>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    spend_key_array: JByteArray<'local>,
    index: jint,
) -> jobject {
    // Convert JByteArray to Vec<u8>
    let result = (|| -> Result<AddressData, ProofError> {
        let spend_key_bytes = env
            .convert_byte_array(&spend_key_array)
            .map_err(|_| ProofError::InvalidKey)?;

        PROOF_MANAGER
            .lock()
            .unwrap()
            .generate_address(spend_key_bytes, index as u32)
    })();

    match result {
        Ok(address) => {
            let hash_map_class = env
                .find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            let hash_map = env
                .new_object(hash_map_class, "()V", &[])
                .expect("Failed to create HashMap");

            // Helper function to put bytes into HashMap
            let mut put_bytes = |key: &str, bytes: &[u8]| {
                let j_key = env.new_string(key).expect("Failed to create string");
                let j_value = env
                    .byte_array_from_slice(bytes)
                    .expect("Failed to create byte array");

                env.call_method(
                    &hash_map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[
                        JValue::Object(&j_key.into()),
                        JValue::Object(&j_value.into()),
                    ],
                )
                .expect("Failed to call put");
            };

            // Add address components to HashMap
            put_bytes("diversifier", &address.diversifier);
            put_bytes("transmissionKey", &address.transmission_key);
            put_bytes("clueKey", &address.clue_key);

            hash_map.into_raw()
        }
        Err(e) => {
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            let hash_map_class = env
                .find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create empty HashMap")
                .into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_createNoteNative<
    'local,
>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    debtor_address: JObject<'local>,
    creditor_address: JObject<'local>,
    amount: jlong,
    asset_id: jlong,
) -> jobject {
    // Helper function to get address data from Java HashMap
    let mut get_address_data = |addr_obj: JObject| -> Result<AddressData, jni::errors::Error> {
        let mut get_bytes = |key: &str| -> Result<Vec<u8>, jni::errors::Error> {
            let j_key = env.new_string(key)?;
            let bytes = env
                .call_method(
                    &addr_obj,
                    "get",
                    "(Ljava/lang/Object;)Ljava/lang/Object;",
                    &[JValue::Object(&j_key.into())],
                )?
                .l()?;

            // Convert to JByteArray first
            let byte_array = JByteArray::from(bytes);
            env.convert_byte_array(&byte_array)
        };

        Ok(AddressData {
            diversifier: get_bytes("diversifier")?,
            transmission_key: get_bytes("transmissionKey")?,
            clue_key: get_bytes("clueKey")?,
        })
    };

    let result = (|| -> Result<Note, ProofError> {
        let debtor = get_address_data(debtor_address).map_err(|_| ProofError::InvalidKey)?;
        let creditor = get_address_data(creditor_address).map_err(|_| ProofError::InvalidKey)?;

        // Convert raw addresses to Penumbra Address type
        let debtor_addr = Address::from_components(
            Diversifier(
                debtor
                    .diversifier
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
            ka::Public(
                debtor
                    .transmission_key
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
            fmd::ClueKey(
                debtor
                    .clue_key
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
        )
        .ok_or_else(|| ProofError::InvalidKey)?;

        let creditor_addr = Address::from_components(
            Diversifier(
                creditor
                    .diversifier
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
            ka::Public(
                creditor
                    .transmission_key
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
            fmd::ClueKey(
                creditor
                    .clue_key
                    .clone()
                    .try_into()
                    .map_err(|_| ProofError::InvalidKey)?,
            ),
        )
        .ok_or_else(|| ProofError::InvalidKey)?;

        // Create Value with amount and asset_id
        let value = Value {
            amount: Amount::from(amount as u64),
            asset_id: asset::Id(Fq::from(asset_id as u64)),
        };

        // Generate random rseed
        let mut rng = rand::thread_rng();
        let mut rseed_bytes = [0u8; 32];
        rng.fill_bytes(&mut rseed_bytes);

        // Create the note using the Note::from_parts constructor
        let note =
            crate::note::Note::from_parts(debtor_addr, creditor_addr, value, Rseed(rseed_bytes))
                .map_err(|e| ProofError::NoteError(e.to_string()))?;

        // Get the commitment
        let commitment = note.commit().0.to_bytes();

        // Return the note data in our FFI-friendly format
        Ok(Note {
            debtor_address: debtor,
            creditor_address: creditor,
            amount: amount as u64,
            asset_id: asset_id as u64,
            commitment: commitment.to_vec(),
        })
    })();

    match result {
        Ok(note) => {
            let hash_map_class = env
                .find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            let hash_map = env
                .new_object(hash_map_class, "()V", &[])
                .expect("Failed to create HashMap");

            let mut put_bytes = |key: &str, bytes: &[u8]| {
                let j_key = env.new_string(key).expect("Failed to create string");
                let j_value = env
                    .byte_array_from_slice(bytes)
                    .expect("Failed to create byte array");

                env.call_method(
                    &hash_map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[
                        JValue::Object(&j_key.into()),
                        JValue::Object(&j_value.into()),
                    ],
                )
                .expect("Failed to call put");
            };

            // Add note data to HashMap
            put_bytes("commitment", &note.commitment);

            // Add addresses
            let mut put_address = |prefix: &str, addr: &AddressData| {
                put_bytes(&format!("{}Diversifier", prefix), &addr.diversifier);
                put_bytes(
                    &format!("{}TransmissionKey", prefix),
                    &addr.transmission_key,
                );
                put_bytes(&format!("{}ClueKey", prefix), &addr.clue_key);
            };

            put_address("debtor", &note.debtor_address);
            put_address("creditor", &note.creditor_address);

            hash_map.into_raw()
        }
        Err(e) => {
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            let hash_map_class = env
                .find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create empty HashMap")
                .into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_signNoteNative<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    seed_phrase: JString<'local>,
    note_obj: JObject<'local>,
) -> jobject {
    rust_log!("signNoteNative called");

    let seed_phrase: String = match env.get_string(&seed_phrase) {
        Ok(s) => s.into(),
        Err(e) => {
            rust_log!("Error getting seed phrase: {:?}", e);
            return create_empty_map(&mut env);
        }
    };

    fn create_empty_map(env: &mut JNIEnv) -> jobject {
      rust_log!("Creating empty map");
        let hash_map_class = env
            .find_class("java/util/HashMap")
            .expect("Failed to find HashMap class");
        env.new_object(hash_map_class, "()V", &[])
            .expect("Failed to create empty HashMap")
            .into_raw()
    }

    // Extract commitment
    let commitment = match get_bytes(&mut env, &note_obj, "commitment") {
        Ok(c) => c,
        Err(e) => {
          rust_log!("Error getting commitment: {:?}", e);
            return create_empty_map(&mut env);
        }
    };

    // Extract addresses
    let debtor_address = match extract_address(&mut env, &note_obj, "debtor") {
        Ok(addr) => addr,
        Err(e) => {
          rust_log!("Error extracting debtor address: {:?}", e);
            return create_empty_map(&mut env);
        }
    };

    let creditor_address = match extract_address(&mut env, &note_obj, "creditor") {
        Ok(addr) => addr,
        Err(e) => {
          rust_log!("Error extracting creditor address: {:?}", e);
            return create_empty_map(&mut env);
        }
    };

    let note = Note {
        commitment: commitment.clone(),
        debtor_address: debtor_address.clone(),
        creditor_address: creditor_address.clone(),
        amount: 0,
        asset_id: 0,
    };

    match PROOF_MANAGER.lock().unwrap().sign_note(seed_phrase, note) {
        Ok(signed_note) => {
            let hash_map_class = env
                .find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            let hash_map = env
                .new_object(hash_map_class, "()V", &[])
                .expect("Failed to create HashMap");

            let mut put_bytes = |key: &str, bytes: &[u8]| {
              rust_log!("Adding {} to result map, length: {}", key, bytes.len());
                let j_key = env.new_string(key).expect("Failed to create string");
                let j_value = env
                    .byte_array_from_slice(bytes)
                    .expect("Failed to create byte array");

                env.call_method(
                    &hash_map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[
                        JValue::Object(&j_key.into()),
                        JValue::Object(&j_value.into()),
                    ],
                )
                .expect("Failed to call put");
            };

            // Add signature and verification key
            put_bytes("signature", &signed_note.signature);
            put_bytes("verificationKey", &signed_note.verification_key);

            // Add note data
            put_bytes("noteCommitment", &commitment);

            // Add debtor address
            put_bytes("debtorDiversifier", &debtor_address.diversifier);
            put_bytes("debtorTransmissionKey", &debtor_address.transmission_key);
            put_bytes("debtorClueKey", &debtor_address.clue_key);

            // Add creditor address
            put_bytes("creditorDiversifier", &creditor_address.diversifier);
            put_bytes(
                "creditorTransmissionKey",
                &creditor_address.transmission_key,
            );
            put_bytes("creditorClueKey", &creditor_address.clue_key);

            hash_map.into_raw()
        }
        Err(e) => {
          rust_log!("Error signing note: {:?}", e);
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            create_empty_map(&mut env)
        }
    }
}

// Helper function to get bytes from JObject
fn get_bytes(env: &mut JNIEnv, obj: &JObject, key: &str) -> Result<Vec<u8>, jni::errors::Error> {
    let j_key = env.new_string(key)?;
    let bytes_obj = env
        .call_method(
            obj,
            "get",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            &[JValue::Object(&j_key.into())],
        )?
        .l()?;

    let byte_array = JByteArray::from(bytes_obj);
    env.convert_byte_array(&byte_array)
}

// Helper function to extract address data
fn extract_address(
    env: &mut JNIEnv,
    note_obj: &JObject,
    prefix: &str,
) -> Result<AddressData, jni::errors::Error> {
    let mut get_bytes = |key: &str| -> Result<Vec<u8>, jni::errors::Error> {
        let j_key = env.new_string(key)?;
        let obj = env
            .call_method(
                note_obj,
                "get",
                "(Ljava/lang/Object;)Ljava/lang/Object;",
                &[JValue::Object(&j_key.into())],
            )?
            .l()?;

        let byte_array = JByteArray::from(obj);
        env.convert_byte_array(&byte_array)
    };

    Ok(AddressData {
        diversifier: get_bytes(&format!("{}Diversifier", prefix))?,
        transmission_key: get_bytes(&format!("{}TransmissionKey", prefix))?,
        clue_key: get_bytes(&format!("{}ClueKey", prefix))?,
    })
}

// Helper function to put address data into HashMap
fn put_address_to_map(env: &mut JNIEnv, hash_map: &JObject, prefix: &str, address: &AddressData) {
    let mut put_bytes = |key: &str, bytes: &[u8]| {
        let j_key = env.new_string(key).expect("Failed to create string");
        let j_value = env
            .byte_array_from_slice(bytes)
            .expect("Failed to create byte array");

        env.call_method(
            hash_map,
            "put",
            "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
            &[
                JValue::Object(&j_key.into()),
                JValue::Object(&j_value.into()),
            ],
        )
        .expect("Failed to call put");
    };

    put_bytes(&format!("{}Diversifier", prefix), &address.diversifier);
    put_bytes(
        &format!("{}TransmissionKey", prefix),
        &address.transmission_key,
    );
    put_bytes(&format!("{}ClueKey", prefix), &address.clue_key);
}

#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_verifySignatureNative<
    'local,
>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    verification_key: JByteArray<'local>,
    commitment: JByteArray<'local>,
    signature: JByteArray<'local>,
) -> jboolean {
    let convert_array = |array: JByteArray<'local>| -> Vec<u8> {
        env.convert_byte_array(&array)
            .map(|bytes| bytes.iter().map(|&b| b as u8).collect())
            .unwrap_or_default()
    };

    let verification_key_bytes = convert_array(verification_key);
    let commitment_bytes = convert_array(commitment);
    let signature_bytes = convert_array(signature);

    match PROOF_MANAGER.lock().unwrap().verify_signature(
        verification_key_bytes,
        commitment_bytes,
        signature_bytes,
    ) {
        Ok(result) => {
            if result {
                1
            } else {
                0
            }
        }
        Err(e) => {
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_createIntentActionNative(
    mut env: JNIEnv,
    _class: JClass,
    debtor_seed_phase: JByteArray,
    rseed_randomness: JByteArray,
    debtor_index: jint,
    creditor_addr: JString,
    amount: jlong,
    asset_id: jlong,
) -> jstring {
  rust_log!("createIntentActionNative: Starting execution");
    
    let result = (|| -> Result<String, ProofError> {
        let debtor_seed = env.convert_byte_array(&debtor_seed_phase)
            .map_err(|e| {
              rust_log!("Failed to convert debtor_seed: {:?}", e);
                ProofError::InvalidKey
            })?;
            
        let rseed = env.convert_byte_array(&rseed_randomness)
            .map_err(|e| {
              rust_log!("Failed to convert rseed: {:?}", e);
                ProofError::InvalidKey
            })?;
            
        let creditor = env.get_string(&creditor_addr)
            .map_err(|e| {
              rust_log!("Failed to get creditor string: {:?}", e);
                ProofError::InvalidKey
            })?
            .into();

            rust_log!("Parameters converted successfully");
            rust_log!("Creditor address: {}", creditor);
        
        let manager = PROOF_MANAGER.try_lock()
            .map_err(|e| {
              rust_log!("Failed to acquire lock: {:?}", e);
                ProofError::IntentError("Failed to acquire lock".into())
            })?;

        manager.create_intent_action(
            debtor_seed,
            rseed,
            debtor_index as u32,
            creditor,
            amount as u64,
            asset_id as u64,
        ).map_err(|e| {
          rust_log!("create_intent_action failed: {:?}", e);
            e
        })
    })();

    match result {
        Ok(json) => {
          rust_log!("Success: Generated intent action");
            match env.new_string(&json) {
                Ok(s) => s.into_raw(),
                Err(e) => {
                  rust_log!("Failed to create new string: {:?}", e);
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
          rust_log!("Error creating intent action: {:?}", e);
            let err_msg = e.to_string();
            let _ = env.throw_new("java/lang/Exception", &err_msg);
            std::ptr::null_mut()
        }
    }
}