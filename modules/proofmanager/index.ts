import ProofManagerModule from './src/ProofManagerModule';
import type { 
  ProofManagerInterface, 
  KeyPair, 
  AddressData, 
  Note, 
  SignedNote,
  NoteCreateParams,  // Make sure this is added
  CreateIntentActionParams
} from './src/ProofManager.types';

export const {
  generateKeys,
  generateAddress,
  createNote,
  signNote,
  verifySignature,
  createIntentAction
} = ProofManagerModule;

export default ProofManagerModule;

export type {
  ProofManagerInterface,
  KeyPair,
  AddressData,
  Note,
  SignedNote,
  NoteCreateParams,
  CreateIntentActionParams    
};