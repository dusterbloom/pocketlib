// lib.rs
uniffi::setup_scaffolding!();

use std::str::FromStr;
use rand::rngs::OsRng;
use std::sync::Arc;
use anyhow::Result;
use ark_groth16::{prepare_verifying_key, r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey};
use base64::prelude::*;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_ff::ToConstraintField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_r1cs_std::prelude::*;
use ark_snark::SNARK;
use once_cell::sync::Lazy;
use decaf377::{
    Bls12_377,
    Fq,

};
use decaf377_fmd as fmd;
use decaf377_ka as ka;
use penumbra_num::Amount;

use rand::{thread_rng, RngCore};

// Re-export and use types from Penumbra
use penumbra_keys::{
    keys::{Diversifier, SeedPhrase, SpendKey, Bip44Path},
    Address,
};
use penumbra_proto::DomainType;
use penumbra_shielded_pool::{
  Note, Rseed
};
use penumbra_tct::StateCommitment;
use penumbra_asset::{Value, asset};
use penumbra_tct::r1cs::StateCommitmentVar;

pub mod note;
pub mod r1cs;

use penumbra_shielded_pool::note::NoteVar;

// Domain separator for note commitments
static NOTECOMMIT_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.notecommit").as_bytes())
});

const GROTH16_PROOF_LENGTH_BYTES: usize = 192;

// Your existing error type for FFI
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ProofError {
    #[error("")]
    InvalidSeed,
    #[error("{error_message}")]
    ProofGenerationFailed { error_message: String },
    #[error("{error_message}")]
    VerificationFailed { error_message: String },
    #[error("{error_message}")]
    InvalidNote { error_message: String },
    #[error("{error_message}")]
    SerializationError { error_message: String },
}

// Simplified FFI types
#[derive(uniffi::Record)]
pub struct SerializedProof {
    pub data: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct AddressInfo {
    pub diversifier: Vec<u8>,
    pub transmission_key: Vec<u8>,
    pub clue_key: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct ProofInput {
    pub seed_phrase: String,
    pub amount: u64,
    pub asset_id: u64,
    pub address_index: u32,
}

// Circuit implementation
#[derive(Clone, Debug)]
pub struct OutputCircuit {
    public: OutputProofPublic,
    private: OutputProofPrivate,
}

impl OutputCircuit {
    fn new(public: OutputProofPublic, private: OutputProofPrivate) -> Self {
        Self { public, private }
    }
}

impl ConstraintSynthesizer<Fq> for OutputCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Note: In the allocation of the address on `NoteVar`, we check the diversified base is not identity.
        let note_var = NoteVar::new_witness(cs.clone(), || Ok(self.private.note.clone()))?;

        // Public inputs
        let claimed_note_commitment =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.note_commitment))?;

        // Note commitment integrity
        let note_commitment = note_var.commit()?;
        note_commitment.enforce_equal(&claimed_note_commitment)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct OutputProofPublic {
    pub note_commitment: StateCommitment,
}

#[derive(Clone, Debug)]
pub struct OutputProofPrivate {
    pub note: Note,
}

#[derive(Clone, Debug)]
pub struct OutputProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl OutputProof {
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: OutputProofPublic,
        private: OutputProofPrivate,
    ) -> anyhow::Result<Self> {
        let circuit = OutputCircuit::new(public, private);
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )?;
        
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        proof.serialize_compressed(&mut proof_bytes[..])?;
        Ok(Self(proof_bytes))
    }

    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: OutputProofPublic,
    ) -> anyhow::Result<()> {
        let proof = Proof::deserialize_compressed(&self.0[..])?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(
            public
                .note_commitment
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("note commitment is not a valid field element"))?,
        );

        Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )?.then_some(()).ok_or_else(|| anyhow::anyhow!("proof did not verify"))
    }
}

// Main proof manager object with FFI interface
#[derive(uniffi::Object)]
pub struct ProofManager {
    proving_key: ProvingKey<Bls12_377>,
    verifying_key: PreparedVerifyingKey<Bls12_377>,
}

#[uniffi::export]
impl ProofManager {
    #[uniffi::constructor]
    fn new() -> Result<Arc<Self>, ProofError> {
        // Create a basic address for initialization
        let diversifier_bytes = [0u8; 16];
        let tx_key_bytes = [0u8; 32];
        let clue_key_bytes = [0u8; 32];
        
        let diversifier = Diversifier(diversifier_bytes);
        let pk_d = ka::Public(tx_key_bytes);
        let clue_key = fmd::ClueKey(clue_key_bytes);
        
        let address = Address::from_components(diversifier, pk_d, clue_key)
        .ok_or_else(|| ProofError::ProofGenerationFailed { 
            error_message: "Failed to create address".to_string() 
        })?;

        // Create a simple note for initialization
        let value = Value {
            amount: Amount::from(0u64),
            asset_id: asset::Id(Fq::ZERO),
        };
        let rseed = Rseed([0u8; 32]);

        let note = Note::from_parts(
            address.clone(),
            value,
            rseed,
        ).map_err(|e| ProofError::ProofGenerationFailed {
            error_message: format!("Failed to create initial note: {}", e)
        })?;

        // Create circuit with these values
        let public = OutputProofPublic {
            note_commitment: note.commit(),
        };
        let private = OutputProofPrivate { note };
        let circuit = OutputCircuit::new(public, private);
        
        // Generate the keys
        let params = generate_setup_params(circuit)?;
        
        Ok(Arc::new(Self {
            proving_key: params.0,
            verifying_key: params.1,
        }))
    }

    fn generate_address(&self, seed_phrase: String, index: u32) -> Result<AddressInfo, ProofError> {
         // Create a cryptographically secure RNG
        let mut rng = OsRng;
        
        // Generate or parse seed phrase based on input
        let seed = if seed_phrase.is_empty() {
            SeedPhrase::generate(&mut rng)
        } else {
            SeedPhrase::from_str(&seed_phrase)
                .map_err(|_| ProofError::InvalidSeed)?
        };
            
        let sk = SpendKey::from_seed_phrase_bip44(seed, &Bip44Path::new(index));
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (address, _dtk) = ivk.payment_address(index.into());

        Ok(AddressInfo {
            diversifier: address.diversifier().0.to_vec(),
            transmission_key: address.transmission_key().0.to_vec(),
            clue_key: address.clue_key().0.to_vec(),
        })
    }

    fn create_proof(&self, input: ProofInput) -> Result<SerializedProof, ProofError> {
        let mut rng = thread_rng();
        let blinding_r = Fq::rand(&mut rng);
        let blinding_s = Fq::rand(&mut rng);

        let address = self.generate_address(input.seed_phrase, input.address_index)?;
        let note = self.create_note(address, input.amount, input.asset_id)?;
        
        let public = OutputProofPublic {
            note_commitment: note.commit(),
        };
        let private = OutputProofPrivate { note };

        let proof = OutputProof::prove(
            blinding_r,
            blinding_s,
            &self.proving_key,
            public,
            private,
        ).map_err(|e| ProofError::ProofGenerationFailed {
            error_message: e.to_string(),
        })?;

        Ok(SerializedProof {
            data: proof.0.to_vec(),
        })
    }

    fn verify_proof(
        &self,
        proof: SerializedProof,
        commitment: Vec<u8>,
    ) -> Result<bool, ProofError> {
        let proof_bytes: [u8; GROTH16_PROOF_LENGTH_BYTES] = proof.data.try_into()
            .map_err(|_| ProofError::SerializationError {
                error_message: "Invalid proof length".to_string(),
            })?;
            
        let proof = OutputProof(proof_bytes);
        
          // Convert Vec<u8> to [u8; 32] for StateCommitment
        let commitment_bytes: [u8; 32] = commitment.try_into()
        .map_err(|_| ProofError::SerializationError {
            error_message: "Invalid commitment length".to_string(),
        })?;
    
        let note_commitment = StateCommitment::try_from(commitment_bytes)
            .map_err(|_| ProofError::SerializationError {
                error_message: "Invalid commitment format".to_string(),
            })?;
            
        let public = OutputProofPublic { note_commitment };

        proof.verify(&self.verifying_key, public)
            .map_err(|e| ProofError::VerificationFailed {
                error_message: e.to_string(),
            })?;

        Ok(true)
    }

    // Helper methods
    fn debug_proof(&self, proof: SerializedProof) -> Result<String, ProofError> {
        Ok(BASE64_STANDARD.encode(&proof.data))
    }

    fn debug_commitment(&self, commitment: Vec<u8>) -> Result<String, ProofError> {
        Ok(BASE64_STANDARD.encode(&commitment))
    }
}

// Private helper functions
impl ProofManager {
    fn create_note(
        &self,
        address: AddressInfo,
        amount: u64,
        asset_id: u64,
    ) -> Result<Note, ProofError> {
        let diversifier = Diversifier(address.diversifier.try_into().map_err(|_| ProofError::InvalidNote {
            error_message: "Invalid diversifier length".to_string(),
        })?);
        
        let pk_d = ka::Public(address.transmission_key.try_into().map_err(|_| ProofError::InvalidNote {
            error_message: "Invalid transmission key length".to_string(),
        })?);
        
        let clue_key = fmd::ClueKey(address.clue_key.try_into().map_err(|_| ProofError::InvalidNote {
            error_message: "Invalid clue key length".to_string(),
        })?);

        let address = Address::from_components(diversifier, pk_d, clue_key)
        .ok_or_else(|| ProofError::InvalidNote {
            error_message: "Failed to create address from components".to_string(),
        })?;

        let value = Value {
            amount: Amount::from(amount),
            asset_id: asset::Id(Fq::from(asset_id)),
        };

        let mut rng = thread_rng();
        let mut rseed_bytes = [0u8; 32];
        rng.fill_bytes(&mut rseed_bytes);
        let rseed = Rseed(rseed_bytes);

        Note::from_parts(
            address.clone(),
            value,
            rseed,
        ).map_err(|e| ProofError::InvalidNote {
            error_message: e.to_string(),
        })
    }
}

fn generate_setup_params(
    circuit: OutputCircuit,
) -> Result<(ProvingKey<Bls12_377>, PreparedVerifyingKey<Bls12_377>), ProofError> {
    let mut rng = thread_rng();
    
    let (pk, vk) = Groth16::<Bls12_377>::circuit_specific_setup(circuit, &mut rng)
        .map_err(|e| ProofError::ProofGenerationFailed {
            error_message: format!("Failed to generate setup parameters: {}", e),
        })?;

    let pvk = prepare_verifying_key(&vk);
    
    Ok((pk, pvk))
}

// Protocol buffer implementations
impl DomainType for OutputProof {
    type Proto = penumbra_proto::penumbra::core::component::shielded_pool::v1::ZkOutputProof;
}

impl From<OutputProof> for penumbra_proto::penumbra::core::component::shielded_pool::v1::ZkOutputProof {
    fn from(proof: OutputProof) -> Self {
        Self {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<penumbra_proto::penumbra::core::component::shielded_pool::v1::ZkOutputProof> for OutputProof {
    type Error = anyhow::Error;

    fn try_from(proto: penumbra_proto::penumbra::core::component::shielded_pool::v1::ZkOutputProof) -> Result<Self, Self::Error> {
        Ok(OutputProof(proto.inner[..].try_into()?))
    }
}