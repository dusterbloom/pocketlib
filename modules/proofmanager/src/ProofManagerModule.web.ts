import { registerWebModule, NativeModule } from 'expo';

import { ChangeEventPayload } from './ProofManager.types';

type ProofManagerModuleEvents = {
  onChange: (params: ChangeEventPayload) => void;
}

class ProofManagerModule extends NativeModule<ProofManagerModuleEvents> {
  PI = Math.PI;
  async setValueAsync(value: string): Promise<void> {
    this.emit('onChange', { value });
  }
  hello() {
    return 'Hello world! 👋';
  }
};

export default registerWebModule(ProofManagerModule);
