import { requireNativeModule } from 'expo-modules-core';
import { ProofManagerInterface } from './ProofManager.types';

// It loads the native module object from the JSI or falls back to
// the bridge module (from NativeModulesProxy) if the remote debugger is on.
export default requireNativeModule<ProofManagerInterface>('proofmanager');