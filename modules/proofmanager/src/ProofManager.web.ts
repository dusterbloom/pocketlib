import { ProofManagerInterface, ProofInput, SerializedProof, AddressInfo } from './ProofManager.types';

class WebProofManager implements ProofManagerInterface {
  async generateAddress(seed_phrase: string, index: number): Promise<AddressInfo> {
    throw new Error('ProofManager is not supported on web platform');
  }

  async createProof(input: ProofInput): Promise<SerializedProof> {
    throw new Error('ProofManager is not supported on web platform');
  }

  async verifyProof(proof: SerializedProof, commitment: number[]): Promise<boolean> {
    throw new Error('ProofManager is not supported on web platform');
  }

  async debugProof(proof: SerializedProof): Promise<string> {
    throw new Error('ProofManager is not supported on web platform');
  }

  async debugCommitment(commitment: number[]): Promise<string> {
    throw new Error('ProofManager is not supported on web platform');
  }
}

export default new WebProofManager();