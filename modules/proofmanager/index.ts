import { requireNativeModule } from 'expo-modules-core';

const ProofManagerModule = requireNativeModule('ProofManager');

export const { 
  generateAddress,
  createProof,
  verifyProof
} = ProofManagerModule;

export default ProofManagerModule;