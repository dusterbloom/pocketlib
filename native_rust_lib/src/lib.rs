// lib.rs

use cosmwasm_std::{Binary, HexBinary};
use jni::objects::{JByteArray, JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jint, jlong, jobject};
use jni::JNIEnv;
use once_cell::sync::Lazy;
use penumbra_keys::keys::{Diversifier, SpendKeyBytes};
use serde::Serialize;
use std::sync::{Arc, Mutex};

pub type Ciphertext = Binary;
pub type CommitmentHash = HexBinary;
pub type NullifierHash = HexBinary;
pub type MerkleRoot = Binary;
pub type Proof = HexBinary;

// Global ProofManager instance
static PROOF_MANAGER: Lazy<Mutex<Arc<ProofManager>>> =
    Lazy::new(|| Mutex::new(ProofManager::new().expect("Failed to initialize ProofManager")));

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

        PROOF_MANAGER
            .lock()
            .unwrap()
            .create_note(debtor, creditor, amount as u64, asset_id as u64)
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
    let seed_phrase: String = env
        .get_string(&seed_phrase)
        .expect("Couldn't get java string!")
        .into();

    // Helper function to get note from Java HashMap
    let mut get_note = |note_obj: JObject| -> Result<Note, jni::errors::Error> {
        let mut get_bytes = |key: &str| -> Result<Vec<u8>, jni::errors::Error> {
            let j_key = env.new_string(key)?;
            let bytes = env
                .call_method(
                    &note_obj,
                    "get",
                    "(Ljava/lang/Object;)Ljava/lang/Object;",
                    &[JValue::Object(&j_key.into())],
                )?
                .l()?;

            // Convert to JByteArray first
            let byte_array = JByteArray::from(bytes);
            env.convert_byte_array(&byte_array)
        };

        let mut get_address = |prefix: &str| -> Result<AddressData, jni::errors::Error> {
            Ok(AddressData {
                diversifier: get_bytes(&format!("{}Diversifier", prefix))?,
                transmission_key: get_bytes(&format!("{}TransmissionKey", prefix))?,
                clue_key: get_bytes(&format!("{}ClueKey", prefix))?,
            })
        };

        Ok(Note {
            debtor_address: get_address("debtor")?,
            creditor_address: get_address("creditor")?,
            amount: 0, // These values are not needed for signing
            asset_id: 0,
            commitment: get_bytes("commitment")?,
        })
    };

    let result = (|| -> Result<SignedNote, ProofError> {
        let note = get_note(note_obj).map_err(|_| ProofError::InvalidKey)?;

        PROOF_MANAGER.lock().unwrap().sign_note(seed_phrase, note)
    })();

    match result {
        Ok(signed_note) => {
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

            put_bytes("signature", &signed_note.signature);
            put_bytes("verificationKey", &signed_note.verification_key);

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

uniffi::setup_scaffolding!();

use decaf377::{Fq, Fr};
use decaf377_fmd as fmd;
use decaf377_ka as ka;
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_asset::{asset, Value};
use penumbra_keys::{
    keys::{Bip44Path, SeedPhrase, SpendKey},
    Address,
};
use penumbra_num::Amount;
use penumbra_shielded_pool::Rseed;
use std::str::FromStr;

use rand::RngCore;

// Our custom note implementation:
mod note;

// Core FFI Types
#[derive(uniffi::Record)]
pub struct AddressData {
    pub diversifier: Vec<u8>,
    pub transmission_key: Vec<u8>,
    pub clue_key: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct KeyPair {
    pub spend_key: Vec<u8>,
    pub view_key: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct Note {
    pub debtor_address: AddressData,
    pub creditor_address: AddressData,
    pub amount: u64,
    pub asset_id: u64,
    pub commitment: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct SignedNote {
    pub note: Note,
    pub signature: Vec<u8>,
    pub verification_key: Vec<u8>,
}

// Core Error Type
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
}

#[derive(Serialize)]
pub struct CreateIntentAction {
    pub note_commitment: HexBinary, // COMMITMENT(AscertainedNote)
    pub auth_sig: HexBinary,
    pub rk: HexBinary,
    pub zkp: HexBinary,
    pub ciphertexts: [Binary; 2], // [0] => Counterparties; [1] => Solver
}

#[derive(uniffi::Object)]
pub struct ProofManager {
    // Constant for spend auth as in test.rs
    spend_auth_randomizer: Fr,
}

#[uniffi::export]
impl ProofManager {
    #[uniffi::constructor]
    pub fn new() -> Result<Arc<Self>, ProofError> {
        Ok(Arc::new(Self {
            spend_auth_randomizer: Fr::from(1u64),
        }))
    }

    // Key Generation
    fn generate_keys(&self, seed_phrase: String) -> Result<KeyPair, ProofError> {
        let seed = SeedPhrase::from_str(&seed_phrase).map_err(|_| ProofError::InvalidSeed)?;

        let spend_key = SpendKey::from_seed_phrase_bip44(seed, &Bip44Path::new(0));
        let view_key = spend_key.full_viewing_key();

        Ok(KeyPair {
            spend_key: spend_key.to_bytes().0.to_vec(),
            view_key: view_key.nullifier_key().0.to_bytes().to_vec(), // Not sure about this
        })
    }

    // Address Generation
    fn generate_address(
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

    pub fn create_intent_action(
        &self,
        debtor_seed_phase: Vec<u8>,
        rseed_randomness: Vec<u8>,
        debtor_index: u32,
        creditor_addr: String,
    ) -> Result<String, ProofError> {
        use cosmwasm_std::HexBinary;

        const TEST_ASSET_ID: u64 = 1;

        let sk_debtor = {
            let seed_phrase = SeedPhrase::from_randomness(&debtor_seed_phase);
            SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0))
        };

        let debtor_addr = {
            let fvk = sk_debtor.full_viewing_key();
            let ivk = fvk.incoming();
            ivk.payment_address(debtor_index.into()).0
        };

        let spend_auth_randomizer = Fr::from(1u64);
        let rsk_debtor = sk_debtor.spend_auth_key().randomize(&spend_auth_randomizer);
        let rk_debtor: VerificationKey<SpendAuth> = rsk_debtor.into();

        let value = Value {
            amount: Amount::from(30u64),
            asset_id: asset::Id(Fq::from(TEST_ASSET_ID)),
        };
        let note = arkworks_gramine::note::Note::from_parts(
            debtor_addr.clone(),
            creditor_addr.parse().map_err(|_| ProofError::InvalidKey)?,
            value,
            Rseed(
                rseed_randomness[..32]
                    .try_into()
                    .expect("rseed_randomness must be at least 32 bytes"),
            ),
        )
        .expect("should be able to create note");

        let note_commitment_1 = {
            let nc = note.commit();
            let bytes = <[u8; 32]>::from(nc);
            HexBinary::from(bytes)
        };
        let auth_sig_1 = {
            let sig = rsk_debtor.sign(rand::thread_rng(), note_commitment_1.as_slice());
            let bytes = Vec::<u8>::from(sig);
            HexBinary::from(bytes)
        };
        let rk_1 = HexBinary::from(rk_debtor.to_bytes());

        let create_intent_action = CreateIntentAction {
            note_commitment: note_commitment_1,
            auth_sig: auth_sig_1,
            rk: rk_1,
            zkp: Default::default(),
            ciphertexts: [
                Binary::new(serde_json::to_vec(&note).unwrap()),
                Default::default(),
            ],
        };

        Ok(serde_json::to_string(&create_intent_action)
            .map_err(|_| ProofError::InvalidKey)?)
    }

    // Create Note
    pub fn create_note(
        &self,
        debtor_address: AddressData,
        creditor_address: AddressData,
        amount: u64,
        asset_id: u64,
    ) -> Result<Note, ProofError> {
        // Clone the data before conversion
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

        let note =
            crate::note::Note::from_parts(debtor_addr, creditor_addr, value, Rseed(rseed_bytes))
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

    // Sign Note
    pub fn sign_note(&self, seed_phrase: String, note: Note) -> Result<SignedNote, ProofError> {
        // Generate spend key from seed phrase
        let seed = SeedPhrase::from_str(&seed_phrase).map_err(|_| ProofError::InvalidSeed)?;
        let spend_key = SpendKey::from_seed_phrase_bip44(seed, &Bip44Path::new(0));

        // Create randomized spend auth key
        let rsk = spend_key
            .spend_auth_key()
            .randomize(&self.spend_auth_randomizer);
        let rk: VerificationKey<SpendAuth> = rsk.into();

        // Sign the commitment
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

    // Verify signature
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

        // Map the verification result to a bool
        Ok(rk.verify(&commitment, &sig).is_ok())
    }
}

// // Tests
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_full_flow() -> Result<(), CryptoError> {
//         let manager = CryptoManager::new()?;

//         // Generate keys
//         let keys = manager.generate_keys(
//             "test test test test test test test test test test test junk".to_string()
//         )?;

//         // Generate addresses
//         let debtor_address = manager.generate_address(keys.spend_key.clone(), 1)?;
//         let creditor_keys = manager.generate_keys(
//             "word word word word word word word word word word word word".to_string()
//         )?;
//         let creditor_address = manager.generate_address(creditor_keys.spend_key, 1)?;

//         // Create and sign note
//         let note = manager.create_note(
//             debtor_address,
//             creditor_address,
//             30u64,
//             1u64,
//         )?;

//         let spendkey = keys.spend_key.

//         let signed = manager.sign_note(, note)?;

//         // Verify signature
//         assert!(manager.verify_signature(
//             signed.verification_key,
//             signed.note.commitment,
//             signed.signature,
//         )?);

//         Ok(())
//     }
// }
