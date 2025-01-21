import { Platform } from 'expo-modules-core';
import ProofManagerModule from './ProofManagerModule';
import { ProofManagerInterface, ProofInput, SerializedProof, AddressInfo } from './ProofManager.types';

// Export the module for web
const ProofManager: ProofManagerInterface = Platform.select({
  web: () => require('./ProofManager.web').default,
  default: () => ProofManagerModule,
})();

export default ProofManager;

// Export types
export type {
  ProofManagerInterface,
  ProofInput,
  SerializedProof,
  AddressInfo,
};