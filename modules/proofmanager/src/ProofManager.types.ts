// ProofManager.types.ts
export interface ProofInput {
  seedPhrase: string;
  amount: number;
  assetId: number;
  addressIndex: number;
}

export interface ProofResult {
  proof: number[];
  commitment: number[];
}

export interface AddressInfo {
  diversifier: number[];
  transmissionKey: number[];
  clueKey: number[];
}

export interface IntentAction {
  noteCommitment: number[];
  authSig: number[];
  rk: number[];
  zkp: number[];
  noteCiphertext: number[];
  auxCiphertext: number[];
}

export interface ProofManagerInterface {
  createProof(input: ProofInput): Promise<ProofResult>;
  verifyProof(proof: number[], commitment: number[]): Promise<boolean>;
  generateAddress(seedPhrase: string, index: number): Promise<AddressInfo>;

  createIntentAction(
    seedPhrase: string,
    amount: number,
    assetId: number,
    addressIndex: number,
    creditorAddress: AddressInfo
): Promise<IntentAction>;

    verifyIntentAction(
        action: IntentAction
    ): Promise<boolean>;
}