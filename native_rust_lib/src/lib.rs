// lib.rs

macro_rules! log {
    ($($arg:tt)*) => {
        #[cfg(target_os = "android")]
        {
            let log_str = format!($($arg)*);
            // Use android_logger or direct NDK logging
            println!("RUST_LOG: {}", log_str);
        }
    };
}

use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject, JValue, JByteArray};
use jni::sys::{jlong, jint, jboolean, jobject};
use once_cell::sync::Lazy;
use penumbra_keys::keys::{Diversifier, SpendKeyBytes};
use std::clone;
use std::sync::{Arc, Mutex};



// Global ProofManager instance
static PROOF_MANAGER: Lazy<Mutex<Arc<ProofManager>>> = Lazy::new(|| {
    Mutex::new(ProofManager::new().expect("Failed to initialize ProofManager"))
});

#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_generateKeysNative<'local>(
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
            let hash_map_class = env.find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            let hash_map = env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create HashMap");

            // Helper function to add byte array to map
            let mut put_bytes = |key: &str, bytes: &[u8]| {
                let j_key = env.new_string(key)
                    .expect("Failed to create string");
                let j_value = env.byte_array_from_slice(bytes)
                    .expect("Failed to create byte array");

                env.call_method(
                    &hash_map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[JValue::Object(&j_key.into()), JValue::Object(&j_value.into())]
                ).expect("Failed to call put");
            };

            put_bytes("spendKey", &key_pair.spend_key);
            put_bytes("viewKey", &key_pair.view_key);

            hash_map.into_raw()
        },
        Err(e) => {
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            let hash_map_class = env.find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create empty HashMap")
                .into_raw()
        }
    }
}


#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_generateAddressNative<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    spend_key_array: JByteArray<'local>,
    index: jint,
) -> jobject {
    // Convert JByteArray to Vec<u8>
    let result = (|| -> Result<AddressData, ProofError> {
        let spend_key_bytes = env.convert_byte_array(&spend_key_array)
            .map_err(|_| ProofError::InvalidKey)?;

        PROOF_MANAGER.lock().unwrap().generate_address(spend_key_bytes, index as u32)
    })();

    match result {
        Ok(address) => {
            let hash_map_class = env.find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            let hash_map = env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create HashMap");

            // Helper function to put bytes into HashMap
            let mut put_bytes = |key: &str, bytes: &[u8]| {
                let j_key = env.new_string(key)
                    .expect("Failed to create string");
                let j_value = env.byte_array_from_slice(bytes)
                    .expect("Failed to create byte array");

                env.call_method(
                    &hash_map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[JValue::Object(&j_key.into()), JValue::Object(&j_value.into())]
                ).expect("Failed to call put");
            };

            // Add address components to HashMap
            put_bytes("diversifier", &address.diversifier);
            put_bytes("transmissionKey", &address.transmission_key);
            put_bytes("clueKey", &address.clue_key);

            hash_map.into_raw()
        },
        Err(e) => {
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            let hash_map_class = env.find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create empty HashMap")
                .into_raw()
        }
    }
}


#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_createNoteNative<'local>(
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
            let bytes = env.call_method(
                &addr_obj,
                "get",
                "(Ljava/lang/Object;)Ljava/lang/Object;",
                &[JValue::Object(&j_key.into())]
            )?.l()?;
            
    
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
        let debtor = get_address_data(debtor_address)
            .map_err(|_| ProofError::InvalidKey)?;
        let creditor = get_address_data(creditor_address)
            .map_err(|_| ProofError::InvalidKey)?;

        PROOF_MANAGER.lock().unwrap().create_note(
            debtor,
            creditor,
            amount as u64,
            asset_id as u64,
        )
    })();

    match result {
        Ok(note) => {
            let hash_map_class = env.find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            let hash_map = env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create HashMap");

            let mut put_bytes = |key: &str, bytes: &[u8]| {
                let j_key = env.new_string(key)
                    .expect("Failed to create string");
                let j_value = env.byte_array_from_slice(bytes)
                    .expect("Failed to create byte array");

                env.call_method(
                    &hash_map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[JValue::Object(&j_key.into()), JValue::Object(&j_value.into())]
                ).expect("Failed to call put");
            };

            // Add note data to HashMap
            put_bytes("commitment", &note.commitment);
            
            // Add addresses
            let mut put_address = |prefix: &str, addr: &AddressData| {
                put_bytes(&format!("{}Diversifier", prefix), &addr.diversifier);
                put_bytes(&format!("{}TransmissionKey", prefix), &addr.transmission_key);
                put_bytes(&format!("{}ClueKey", prefix), &addr.clue_key);
            };

            put_address("debtor", &note.debtor_address);
            put_address("creditor", &note.creditor_address);

            hash_map.into_raw()
        },
        Err(e) => {
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            let hash_map_class = env.find_class("java/util/HashMap")
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
    log!("signNoteNative called");
    
    let seed_phrase: String = match env.get_string(&seed_phrase) {
        Ok(s) => s.into(),
        Err(e) => {
            log!("Error getting seed phrase: {:?}", e);
            return create_empty_map(&mut env);
        }
    };

    fn create_empty_map(env: &mut JNIEnv) -> jobject {
        log!("Creating empty map");
        let hash_map_class = env.find_class("java/util/HashMap")
            .expect("Failed to find HashMap class");
        env.new_object(hash_map_class, "()V", &[])
            .expect("Failed to create empty HashMap")
            .into_raw()
    }
    

    // Extract commitment
    let commitment = match get_bytes(&mut env, &note_obj, "commitment") {
        Ok(c) => c,
        Err(e) => {
            log!("Error getting commitment: {:?}", e);
            return create_empty_map(&mut env);
        }
    };

    // Extract addresses
    let debtor_address = match extract_address(&mut env, &note_obj, "debtor") {
        Ok(addr) => addr,
        Err(e) => {
            log!("Error extracting debtor address: {:?}", e);
            return create_empty_map(&mut env);
        }
    };

    let creditor_address = match extract_address(&mut env, &note_obj, "creditor") {
        Ok(addr) => addr,
        Err(e) => {
            log!("Error extracting creditor address: {:?}", e);
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
            let hash_map_class = env.find_class("java/util/HashMap")
                .expect("Failed to find HashMap class");
            let hash_map = env.new_object(hash_map_class, "()V", &[])
                .expect("Failed to create HashMap");

            let mut put_bytes = |key: &str, bytes: &[u8]| {
                log!("Adding {} to result map, length: {}", key, bytes.len());
                let j_key = env.new_string(key)
                    .expect("Failed to create string");
                let j_value = env.byte_array_from_slice(bytes)
                    .expect("Failed to create byte array");

                env.call_method(
                    &hash_map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[JValue::Object(&j_key.into()), JValue::Object(&j_value.into())]
                ).expect("Failed to call put");
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
            put_bytes("creditorTransmissionKey", &creditor_address.transmission_key);
            put_bytes("creditorClueKey", &creditor_address.clue_key);

            hash_map.into_raw()
        },
        Err(e) => {
            log!("Error signing note: {:?}", e);
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            create_empty_map(&mut env)
        }
    }
}

// Helper function to get bytes from JObject
fn get_bytes(env: &mut JNIEnv, obj: &JObject, key: &str) -> Result<Vec<u8>, jni::errors::Error> {
    let j_key = env.new_string(key)?;
    let bytes_obj = env.call_method(
        obj,
        "get",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        &[JValue::Object(&j_key.into())]
    )?.l()?;
    
    let byte_array = JByteArray::from(bytes_obj);
    env.convert_byte_array(&byte_array)
}

// #[no_mangle]
// pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_signNoteNative<'local>(
//     mut env: JNIEnv<'local>,
//     _class: JClass<'local>,
//     seed_phrase: JString<'local>,
//     note_obj: JObject<'local>,
// ) -> jobject {
//     log!("signNoteNative called");
    
//     let seed_phrase: String = match env.get_string(&seed_phrase) {
//         Ok(s) => s.into(),
//         Err(e) => {
//             log!("Error getting seed phrase: {:?}", e);
//             return create_empty_map(&mut env);
//         }
//     };

//       // Helper function to create empty HashMap on error
//       fn create_empty_map(env: & mut JNIEnv) -> jobject {
//         log!("Creating empty map due to error");
//         let hash_map_class = env.find_class("java/util/HashMap")
//             .expect("Failed to find HashMap class");
//         env.new_object(hash_map_class, "()V", &[])
//             .expect("Failed to create empty HashMap")
//             .into_raw()
//     }

//     // Helper function to get bytes with logging
//     let mut get_bytes = |key: &str| -> Result<Vec<u8>, jni::errors::Error> {
//         log!("Getting bytes for key: {}", key);
//         let j_key = env.new_string(key)?;
//         let obj = env.call_method(
//             &note_obj,
//             "get",
//             "(Ljava/lang/Object;)Ljava/lang/Object;",
//             &[JValue::Object(&j_key.into())]
//         )?.l()?;
        
//         let byte_array = JByteArray::from(obj);
//         let bytes = env.convert_byte_array(&byte_array)?;
//         log!("Got bytes for {}, length: {}", key, bytes.len());
//         Ok(bytes)
//     };

//     // Extract all the note components
//     let (commitment, debtor_address, creditor_address) = match (
//         get_bytes("commitment"),
//         extract_address(&mut env, &note_obj, "debtor"),
//         extract_address(&mut env, &note_obj, "creditor")
//     ) {
//         (Ok(c), Ok(d), Ok(cr)) => (c, d, cr),
//         _ => return create_empty_map(&mut env)
//     };

//     let note = Note {
//         debtor_address,
//         creditor_address,
//         amount: 0, // These aren't needed for signing
//         asset_id: 0,
//         commitment: commitment.clone(), // Important: keep the commitment
//     };

//     match PROOF_MANAGER.lock().unwrap().sign_note(seed_phrase, note.clone()) {
//         Ok(signed_note) => {
//             let hash_map_class = env.find_class("java/util/HashMap")
//                 .expect("Failed to find HashMap class");
//             let hash_map = env.new_object(hash_map_class, "()V", &[])
//                 .expect("Failed to create HashMap");

//             let mut put_bytes = |key: &str, bytes: &[u8]| {
//                 log!("Adding {} to result map, length: {}", key, bytes.len());
//                 let j_key = env.new_string(key)
//                     .expect("Failed to create string");
//                 let j_value = env.byte_array_from_slice(bytes)
//                     .expect("Failed to create byte array");

//                 env.call_method(
//                     &hash_map,
//                     "put",
//                     "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
//                     &[JValue::Object(&j_key.into()), JValue::Object(&j_value.into())]
//                 ).expect("Failed to call put");
//             };

//             // Add signature and verification key
//             put_bytes("signature", &signed_note.signature);
//             put_bytes("verificationKey", &signed_note.verification_key);
            
//             // Important: Add back the commitment and address data
//             put_bytes("noteCommitment", &commitment);
//             put_address_to_map(&mut env, &hash_map, "debtor", &note.debtor_address);
//             put_address_to_map(&mut env, &hash_map, "creditor", &note.creditor_address);

//             hash_map.into_raw()
//         },
//         Err(e) => {
//             log!("Error signing note: {:?}", e);
//             env.throw_new("java/lang/Exception", e.to_string())
//                 .expect("Failed to throw exception");
//             create_empty_map(&mut env)
//         }
//     }
// }

// Helper function to extract address data
fn extract_address(env: &mut JNIEnv, note_obj: &JObject, prefix: &str) -> Result<AddressData, jni::errors::Error> {
    let mut get_bytes = |key: &str| -> Result<Vec<u8>, jni::errors::Error> {
        let j_key = env.new_string(key)?;
        let obj = env.call_method(
            note_obj,
            "get",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            &[JValue::Object(&j_key.into())]
        )?.l()?;
        
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
        let j_value = env.byte_array_from_slice(bytes).expect("Failed to create byte array");

        env.call_method(
            hash_map,
            "put",
            "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
            &[JValue::Object(&j_key.into()), JValue::Object(&j_value.into())]
        ).expect("Failed to call put");
    };

    put_bytes(&format!("{}Diversifier", prefix), &address.diversifier);
    put_bytes(&format!("{}TransmissionKey", prefix), &address.transmission_key);
    put_bytes(&format!("{}ClueKey", prefix), &address.clue_key);
}



#[no_mangle]
pub extern "system" fn Java_expo_modules_proofmanager_ProofManagerModule_verifySignatureNative<'local>(
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
        Ok(result) => if result { 1 } else { 0 },
        Err(e) => {
            env.throw_new("java/lang/Exception", e.to_string())
                .expect("Failed to throw exception");
            0
        }
    }
}



uniffi::setup_scaffolding!();

use std::str::FromStr;
use decaf377::{Fq, Fr};
use decaf377_rdsa::{SpendAuth, VerificationKey, Signature};
use penumbra_keys::{
    keys::{Bip44Path, SeedPhrase, SpendKey},
    Address,
};
use penumbra_asset::{asset, Value};
use penumbra_num::Amount;
use penumbra_shielded_pool::Rseed;
use decaf377_ka as ka;
use decaf377_fmd as fmd;

use rand::RngCore;

// Our custom note implementation:
mod note;

// Core FFI Types
#[derive(uniffi::Record)]
#[derive(Clone, PartialEq, Eq)]
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
#[derive(Clone, PartialEq, Eq)]

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
        let seed = SeedPhrase::from_str(&seed_phrase)
            .map_err(|_| ProofError::InvalidSeed)?;
        
        let spend_key = SpendKey::from_seed_phrase_bip44(seed, &Bip44Path::new(0));
        let view_key = spend_key.full_viewing_key();

        Ok(KeyPair {
            spend_key: spend_key.to_bytes().0.to_vec(),
            view_key: view_key.nullifier_key().0.to_bytes().to_vec(), // Not sure about this
        })
    }

     // Address Generation
     fn generate_address(&self, spend_key_bytes: Vec<u8>, index: u32) -> Result<AddressData, ProofError> {
        let spend_key_bytes: [u8; 32] = spend_key_bytes.try_into()
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
        Diversifier(debtor_address.diversifier.clone().try_into().map_err(|_| ProofError::InvalidKey)?),
        ka::Public(debtor_address.transmission_key.clone().try_into().map_err(|_| ProofError::InvalidKey)?),
        fmd::ClueKey(debtor_address.clue_key.clone().try_into().map_err(|_| ProofError::InvalidKey)?),
    ).ok_or_else(|| ProofError::InvalidKey)?;

    let creditor_addr = Address::from_components(
        Diversifier(creditor_address.diversifier.clone().try_into().map_err(|_| ProofError::InvalidKey)?),
        ka::Public(creditor_address.transmission_key.clone().try_into().map_err(|_| ProofError::InvalidKey)?),
        fmd::ClueKey(creditor_address.clue_key.clone().try_into().map_err(|_| ProofError::InvalidKey)?),
    ).ok_or_else(|| ProofError::InvalidKey)?;
    

        let value = Value {
            amount: Amount::from(amount),
            asset_id: asset::Id(Fq::from(asset_id)),
        };

        let mut rng = rand::thread_rng();
        let mut rseed_bytes = [0u8; 32];
        rng.fill_bytes(&mut rseed_bytes);

        let note = crate::note::Note::from_parts(
            debtor_addr,
            creditor_addr,
            value,
            Rseed(rseed_bytes),
        ).map_err(|e| ProofError::NoteError(e.to_string()))?;

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
    pub fn sign_note(
        &self,
        seed_phrase: String,
        note: Note,
    ) -> Result<SignedNote, ProofError> {
        // Generate spend key from seed phrase
        let seed = SeedPhrase::from_str(&seed_phrase)
            .map_err(|_| ProofError::InvalidSeed)?;
        let spend_key = SpendKey::from_seed_phrase_bip44(seed, &Bip44Path::new(0));

        // Create randomized spend auth key
        let rsk = spend_key.spend_auth_key().randomize(&self.spend_auth_randomizer);
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

        let sig = Signature::try_from(signature.as_slice())
            .map_err(|_| ProofError::InvalidSignature)?;

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