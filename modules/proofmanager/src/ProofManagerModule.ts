import { NativeModule, requireNativeModule } from 'expo';

import { ProofManagerModuleEvents } from './ProofManager.types';

declare class ProofManagerModule extends NativeModule<ProofManagerModuleEvents> {
  PI: number;
  hello(): string;
  setValueAsync(value: string): Promise<void>;
}

// This call loads the native module object from the JSI.
export default requireNativeModule<ProofManagerModule>('ProofManager');
