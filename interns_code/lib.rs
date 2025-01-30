// lib.rs
use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject, JValue, JByteArray};
use jni::sys::{jlong, jint, jboolean, jobject};
use std::sync::{Arc, Mutex};



use uniffi;
use once_cell::sync::Lazy;
use rand::thread_rng;
use rand::RngCore as _;


// --- Penumbra + decaf377 imports ---
use decaf377::{Fq, Fr};
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_asset::{Value, asset};
use penumbra_num::Amount;
use penumbra_keys::{
    keys::{SeedPhrase, SpendKey, Bip44Path},
    Address,
};
use penumbra_shielded_pool::Rseed;

use bincode; // or any other serializer
use serde::{Serialize, Deserialize};

// Our custom note implementation:
mod note;
use note::Note;

// -------------- UniFFI Setup --------------
// This macro will include the generated scaffolding code
uniffi::setup_scaffolding!();

// -------------- Error types --------------
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum UniFfiError {
    #[error("Failed to deserialize: {error_message}")]
    DeserializationError { error_message: String },
    #[error("Invalid note: {error_message}")]
    InvalidNote { error_message: String },
    #[error("Invalid key: {error_message}")]
    InvalidKey { error_message: String },
}

// -------------- Records to expose --------------

// The final result of “commitment and signature”
#[derive(uniffi::Record, Clone)]
pub struct NoteCreationResult {
    pub note_commitment: Vec<u8>,  // 32 bytes
    pub auth_sig: Vec<u8>,         // The signature
    pub rk: Vec<u8>,               // Randomized verification key
}

// -------------- 1) Key Generation from Seed --------------
/// Generate a SpendKey from 32 bytes of randomness (debtor side).
#[uniffi::export]
pub fn sk_from_seed(seed_randomness: Vec<u8>) -> Vec<u8> {
    let seed_phrase = SeedPhrase::from_randomness(&seed_randomness);
    let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    bincode::serialize(&sk).expect("serialization should succeed")
}

// -------------- 2) Address Generation from a SpendKey --------------
#[uniffi::export]
pub fn addr_from_sk(sk_bytes: Vec<u8>, index: u32) -> Result<Vec<u8>, UniFfiError> {
    // Deserialize the SpendKey
    let sk: SpendKey = bincode::deserialize(&sk_bytes).map_err(|e| UniFfiError::InvalidKey {
        error_message: e.to_string(),
    })?;

    let fvk = sk.full_viewing_key();
    let ivk = fvk.incoming();
    let address = ivk.payment_address(index.into()).0;

    // We’ll just serialize the entire Address struct using bincode
    let serialized_address = bincode::serialize(&address).map_err(|e| UniFfiError::InvalidKey {
        error_message: e.to_string(),
    })?;

    Ok(serialized_address)
}

// -------------- 3) Note Creation --------------
/// Create a Note from raw address bytes (debtor, creditor), amount, asset_id, and an Rseed.
#[uniffi::export]
pub fn create_note(
    debtor_addr_bytes: Vec<u8>,
    creditor_addr_bytes: Vec<u8>,
    amount: u64,
    asset_id: u64,
    rseed: Vec<u8>, // <-- changed from [u8; 32] to Vec<u8>
) -> Result<Vec<u8>, UniFfiError> {
    // 1) Enforce rseed is length 32
    if rseed.len() != 32 {
        return Err(UniFfiError::DeserializationError {
            error_message: "rseed must be 32 bytes".to_string(),
        });
    }

    // 2) Convert Vec<u8> into [u8; 32]
    let mut rseed_array = [0u8; 32];
    rseed_array.copy_from_slice(&rseed[..]);

    // 3) Continue with your logic
    let debtor_addr: Address = bincode::deserialize(&debtor_addr_bytes)
        .map_err(|e| UniFfiError::DeserializationError {
            error_message: format!("debtor address: {}", e),
        })?;

    let creditor_addr: Address = bincode::deserialize(&creditor_addr_bytes)
        .map_err(|e| UniFfiError::DeserializationError {
            error_message: format!("creditor address: {}", e),
        })?;

    let value = Value {
        amount: Amount::from(amount),
        asset_id: asset::Id(Fq::from(asset_id)),
    };

    let note = Note::from_parts(
        debtor_addr,
        creditor_addr,
        value,
        Rseed(rseed_array),
    )
    .map_err(|err| UniFfiError::InvalidNote {
        error_message: err.to_string(),
    })?;

    let note_bytes = bincode::serialize(&note).map_err(|err| UniFfiError::DeserializationError {
        error_message: err.to_string(),
    })?;

    Ok(note_bytes)
}


// -------------- 4) Commitment + Signature --------------
/// Given a Note (as bytes) and the debtor’s SpendKey (as bytes),
/// return (commitment, auth_sig, randomized_key).
#[uniffi::export]
pub fn commitment_and_signature(
    note_bytes: Vec<u8>,
    debtor_sk_bytes: Vec<u8>,
) -> Result<NoteCreationResult, UniFfiError> {
    // Reconstruct the note
    let note: Note = bincode::deserialize(&note_bytes).map_err(|err| UniFfiError::DeserializationError {
        error_message: err.to_string(),
    })?;

    // Reconstruct the spend key
    let sk: SpendKey = bincode::deserialize(&debtor_sk_bytes).map_err(|err| UniFfiError::InvalidKey {
        error_message: err.to_string(),
    })?;

    // Commitment
    let commitment = note.commit(); // -> StateCommitment
    let commitment_bytes = commitment.0.to_bytes(); // [u8; 32]

    // Randomize the spend auth key (if desired)
    // For demonstration, always use 1u64. In practice, might be random or zero.
    let spend_auth_randomizer = Fr::from(1u64);
    let rsk_debtor = sk.spend_auth_key().randomize(&spend_auth_randomizer);
    let rk_debtor: VerificationKey<SpendAuth> = rsk_debtor.into();

    // Sign the 32-byte commitment
    let sig = rsk_debtor.sign(thread_rng(), commitment_bytes.as_ref());
    let sig_bytes = sig.to_bytes().to_vec();

    Ok(NoteCreationResult {
        note_commitment: commitment_bytes.to_vec(),
        auth_sig: sig_bytes,
        rk: rk_debtor.to_bytes().to_vec(), // Randomized debtor key
    })
}
