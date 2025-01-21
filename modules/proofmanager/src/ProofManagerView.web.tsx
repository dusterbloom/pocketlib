import * as React from 'react';

import { ProofManagerViewProps } from './ProofManager.types';

export default function ProofManagerView(props: ProofManagerViewProps) {
  return (
    <div>
      <iframe
        style={{ flex: 1 }}
        src={props.url}
        onLoad={() => props.onLoad({ nativeEvent: { url: props.url } })}
      />
    </div>
  );
}
