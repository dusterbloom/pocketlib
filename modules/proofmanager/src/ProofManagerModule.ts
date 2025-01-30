import { requireNativeModule } from 'expo-modules-core';
import { ProofManagerInterface } from './ProofManager.types';

const ProofManagerModule = requireNativeModule('ProofManager') as ProofManagerInterface;

export default ProofManagerModule;