// ProofManager.types.ts
export interface AddressData {
  diversifier: number[];
  transmissionKey: number[];
  clueKey: number[];
}

export interface KeyPair {
  spendKey: number[];
  viewKey: number[];
}

export interface Note {
  debtorAddress: AddressData;
  creditorAddress: AddressData;
  amount: number;
  assetId: number;
  commitment: number[];
}

export interface NoteCreateParams {
  debtorAddress: AddressData;
  creditorAddress: AddressData;
  amount: number;
  assetId: number;
}

export interface GenerateAddressParams {
  spendKey: number[];
  index: number;
}

export interface SignedNote {
  note: Note;
  signature: number[];
  verificationKey: number[];
}

export interface SignNoteParams {
  seedPhrase: string;
  note: Note;
}


export interface ProofManagerInterface {
  // Key and Address Generation
  generateKeys(seedPhrase: string): Promise<KeyPair>;
  generateAddress(params: GenerateAddressParams): Promise<AddressData>;

  // Note Operations
  createNote(params: NoteCreateParams): Promise<Note>;

  // Signature Operations
  signNote(params: SignNoteParams): Promise<SignedNote>;
  verifySignature(
    verificationKeyBytes: number[],
    commitment: number[],
    signature: number[]
  ): Promise<boolean>;

  createIntentAction(
    debtorSeedPhase: number[],
    rseedRandomness: number[],
    debtorIndex: number,
    creditorAddr: string
  ): Promise<string>;
}