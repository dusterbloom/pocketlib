export interface AddressInfo {
  diversifier: number[];
  transmissionKey: number[];
  clueKey: number[];
}

export interface ProofInput {
  seedPhrase: string;
  amount: number;
  assetId: number;
  addressIndex: number;
}

export type SerializedProof = number[]; // ByteArray comes as number[] from native

export interface ProofManagerInterface {
  createProof(input: ProofInput): Promise<SerializedProof>;
  verifyProof(proof: SerializedProof, commitment: number[]): Promise<boolean>;
  generateAddress(seedPhrase: string, index: number): Promise<AddressInfo>;
}