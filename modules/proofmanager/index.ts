import { requireNativeModule } from 'expo-modules-core';

const ProofManagerModule = requireNativeModule('ProofManager');

export const { 
  generateKeys,
  generateAddress,
  createNote,
  signNote,
  verifySignature
} = ProofManagerModule;

export default ProofManagerModule;