import ProofManagerModule from './src/ProofManagerModule';
import type { 
  ProofManagerInterface, 
  KeyPair, 
  AddressData, 
  Note, 
  SignedNote,
  NoteCreateParams  // Make sure this is added
} from './src/ProofManager.types';

export const {
  generateKeys,
  generateAddress,
  createNote,
  signNote,
  verifySignature
} = ProofManagerModule;

export default ProofManagerModule;

export type {
  ProofManagerInterface,
  KeyPair,
  AddressData,
  Note,
  SignedNote,
  NoteCreateParams  // Add this export
};

export type IntentAction = {
  noteCommitment: string;
  authSig: string;
  rk: string;
  zkp: string;
  ciphertexts: string[];
};

export async function createIntentAction(params: {
  debtorSeedPhase: Uint8Array;
  rseedRandomness: Uint8Array;
  debtorIndex: number;
  creditorAddr: string;
}): Promise<IntentAction> {
  const result = await ProofManagerModule.createIntentAction(
    Array.from(params.debtorSeedPhase),
    Array.from(params.rseedRandomness),
    params.debtorIndex,
    params.creditorAddr
  );
  
  return JSON.parse(result);
}