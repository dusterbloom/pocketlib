import { requireNativeView } from 'expo';
import * as React from 'react';

import { ProofManagerViewProps } from './ProofManager.types';

const NativeView: React.ComponentType<ProofManagerViewProps> =
  requireNativeView('ProofManager');

export default function ProofManagerView(props: ProofManagerViewProps) {
  return <NativeView {...props} />;
}
