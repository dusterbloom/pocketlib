import { requireOptionalNativeModule, EventEmitter, EventSubscription } from 'expo-modules-core';

import ProofManagerModule from './src/ProofManagerModule';

import ProofManagerModuleView from './src/ProofManagerView';
import {
  ChangeEventPayload,
  ProofManagerViewProps,
} from "./src/ProofManager.types";

export const PI = ProofManagerModule.PI;

export function hello(): string {
    return ProofManagerModule.hello(); 
}

export async function rustAdd(a: number, b: number) {
    return await ProofManagerModule.rustAdd(a, b);
}
  
  
export async function setValueAsync(value: string) {
    return await ProofManagerModule.setValueAsync(value);
  }


  
  export function addChangeListener(
    listener: (event: ChangeEventPayload) => void
  ): EventSubscription {
    return ProofManagerModule.addListener("onChange", listener);
  }


export { ProofManagerModuleView, ProofManagerViewProps, ChangeEventPayload };
// Reexport the native module. On web, it will be resolved to ProofManagerModule.web.ts
// and on native platforms to ProofManagerModule.ts
// export { default } from './src/ProofManagerModule';
// export { default as ProofManagerView } from './src/ProofManagerView';
// export * from  './src/ProofManager.types';
