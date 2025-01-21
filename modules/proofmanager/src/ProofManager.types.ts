// src/types.ts
export interface AddressInfo {
  diversifier: number[];
  transmission_key: number[];
  clue_key: number[];
}

export interface ProofInput {
  seed_phrase: string;
  amount: number;
  asset_id: number;
  address_index: number;
}

export interface SerializedProof {
  data: number[];
}

export interface ProofManagerInterface {
  generateAddress(seed_phrase: string, index: number): Promise<AddressInfo>;
  createProof(input: ProofInput): Promise<SerializedProof>;
  verifyProof(proof: SerializedProof, commitment: number[]): Promise<boolean>;
  debugProof(proof: SerializedProof): Promise<string>;
  debugCommitment(commitment: number[]): Promise<string>;
}