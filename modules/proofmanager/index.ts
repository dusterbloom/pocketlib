import { requireNativeModule } from 'expo-modules-core';

const ProofManagerModule = requireNativeModule('ProofManager');

export const { 
  hello,
  rustAdd,
  generateAddress,
  createProof,
  verifyProof
} = ProofManagerModule;

export default ProofManagerModule;