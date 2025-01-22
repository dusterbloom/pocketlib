export function bytesToHex(bytes: number[]): string {
    return bytes.map(b => b.toString(16).padStart(2, '0')).join('');
  }
  

export function addressToHex(address: {
    clueKey: number[];
    diversifier: number[];
    transmissionKey: number[];
  }) {
    return {
      clueKey: bytesToHex(address.clueKey),
      diversifier: bytesToHex(address.diversifier),
      transmissionKey: bytesToHex(address.transmissionKey),
    };
  }
  

  export function convertByteArraysToHex(obj: any): any {
    if (Array.isArray(obj)) {
      // If it's a raw array of numbers, convert to hex
      if (obj.every((item) => typeof item === 'number')) {
        return bytesToHex(obj);
      }
      // Otherwise convert each item recursively
      return obj.map(convertByteArraysToHex);
    } else if (obj !== null && typeof obj === 'object') {
      const result: any = {};
      for (const key in obj) {
        result[key] = convertByteArraysToHex(obj[key]);
      }
      return result;
    } else {
      // If neither an array nor object, return as-is
      return obj;
    }
  }
  