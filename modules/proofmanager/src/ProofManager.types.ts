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

export interface VerifySignatureParams {
  verificationKey: number[];
  commitment: number[];
  signature: number[];
}


export interface CreateIntentActionParams {
  debtorSeedPhrase: number[];
  rseedRandomness: number[];
  debtorIndex: number;
  creditorAddr: string;
  amount: number;
  assetId: number;
}


export interface ProofManagerInterface {
  // Key and Address Generation
  generateKeys(seedPhrase: string): Promise<KeyPair>;
  generateAddress(params: GenerateAddressParams): Promise<AddressData>;

  // Note Operations
  createNote(params: NoteCreateParams): Promise<Note>;

 // Signature Operations
 signNote(params: SignNoteParams): Promise<SignedNote>;
 verifySignature(params: VerifySignatureParams): Promise<boolean>;
 
 
 // Create intent
 createIntentAction(params: CreateIntentActionParams): Promise<string>;
}