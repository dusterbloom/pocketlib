import { Image, StyleSheet, Platform } from 'react-native';
import { HelloWave } from '@/components/HelloWave';
import ParallaxScrollView from '@/components/ParallaxScrollView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';

import * as ProofManager from '@/modules/proofmanager'
import { ProofInput } from '../../modules/proofmanager/src/ProofManager.types';
import { useEffect, useState } from 'react';
import { Text } from "react-native";
import { addressToHex, bytesToHex, convertByteArraysToHex } from '@/utils/hex';

export default function HomeScreen() {
  // State for proof, commitment, etc. in hex
  const [addressHex, setAddressHex] = useState<{
    clueKey: string;
    diversifier: string;
    transmissionKey: string;
  } | null>(null);

  const [proofHex, setProofHex] = useState<string | null>(null);
  const [commitmentHex, setCommitmentHex] = useState<string | null>(null);
  const [isValid, setIsValid] = useState<boolean | null>(null);

  // Error + timings as before
  const [error, setError] = useState<string | null>(null);  
  const [timings, setTimings] = useState<{
    addressGenTime: number | null;
    proofGenTime: number | null;
    verifyTime: number | null;
  }>({
    addressGenTime: null,
    proofGenTime: null,
    verifyTime: null,
  });

  useEffect(() => {
    async function generateProof() {
      try {
        const input: ProofInput = {
          seedPhrase:
            'garage advice weekend this dose mango sign horse tool torch mosquito repeat sentence valid scheme pull punch need prosper build actor say cancel allow',
          amount: 1000,
          assetId: 1,
          addressIndex: 0,
        };

        console.log('Generating proof with input:', input);

        // --- Step 1: generateAddress ---
        const startAddr = Date.now();
        const address = await ProofManager.generateAddress(
          input.seedPhrase,
          input.addressIndex
        );
        const endAddr = Date.now();
        const addressGenTime = endAddr - startAddr;

        // Convert the address fields to hex
        const hexAddr = convertByteArraysToHex(address);
        setAddressHex(hexAddr); // <--- store in state
        console.log('Hex address:', hexAddr);
        console.log(`Address generation took ${addressGenTime}ms`);
        setTimings((prev) => ({ ...prev, addressGenTime }));

        // --- Step 2: createProof ---
        const startProof = Date.now();
        const result = await ProofManager.createProof(input);
        const endProof = Date.now();
        const proofGenTime = endProof - startProof;

        console.log('Generated proof result:', {
          proofLength: result.proof.length,
          commitmentLength: result.commitment.length,
        });
        console.log(`Proof generation took ${proofGenTime}ms`);
        setTimings((prev) => ({ ...prev, proofGenTime }));

        // Convert the proof and commitment arrays to hex
        const proofHexString = convertByteArraysToHex(result.proof);
        const commitmentHexString = convertByteArraysToHex(result.commitment);
        setProofHex(proofHexString);
        setCommitmentHex(commitmentHexString);

        console.log('Generated proof hex result:', {
          proofHexString,
          commitmentHexString,
        });

        // --- Step 3: verifyProof ---
        const startVerify = Date.now();
        const valid = await ProofManager.verifyProof(
          result.proof,
          result.commitment
        );
        const endVerify = Date.now();
        const verifyTime = endVerify - startVerify;

        setIsValid(valid); // <--- store verification result
        console.log('Proof verification result:', valid);
        console.log(`Proof verification took ${verifyTime}ms`);
        setTimings((prev) => ({ ...prev, verifyTime }));

      } catch (err) {
        console.error('Error in proof generation:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
      }
    }

    generateProof();
  }, []);

  return (
    <ParallaxScrollView
      headerBackgroundColor={{ light: '#A1CEDC', dark: '#1D3D47' }}
      headerImage={
        <Image
          source={require('@/assets/images/partial-react-logo.png')}
          style={styles.reactLogo}
        />
      }
    >
      <ThemedView style={styles.titleContainer}>
        <HelloWave />
      </ThemedView>

      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Step 1: Try it</ThemedText>

        {/* Show the proof */}
        <Text>Proof (hex): {proofHex ?? 'Generating...'}</Text>
        <Text>Commitment (hex): {commitmentHex ?? 'Generating...'}</Text>

        {/* Show address hex */}
        {addressHex && (
          <>
            <Text>Address ClueKey (hex): {addressHex.clueKey}</Text>
            <Text>Address Diversifier (hex): {addressHex.diversifier}</Text>
            <Text>Address TransmissionKey (hex): {addressHex.transmissionKey}</Text>
          </>
        )}

        {/* Show verification result */}
        {isValid !== null && (
          <Text>Proof verification: {isValid ? 'Valid ✅' : 'Invalid ❌'}</Text>
        )}

        {error && <Text style={{ color: 'red' }}>Error: {error}</Text>}

        {/* Show timing results */}
        <ThemedText type="subtitle">Timing Results</ThemedText>
        {timings.addressGenTime != null && (
          <Text>Address Generation: {timings.addressGenTime} ms</Text>
        )}
        {timings.proofGenTime != null && (
          <Text>Proof Generation: {timings.proofGenTime} ms</Text>
        )}
        {timings.verifyTime != null && (
          <Text>Proof Verification: {timings.verifyTime} ms</Text>
        )}

        <ThemedText>
          Edit <ThemedText type="defaultSemiBold">app/(tabs)/index.tsx</ThemedText> to see changes.
          Press{' '}
          <ThemedText type="defaultSemiBold">
            {Platform.select({
              ios: 'cmd + d',
              android: 'cmd + m',
              web: 'F12'
            })}
          </ThemedText>{' '}
          to open developer tools.
        </ThemedText>
      </ThemedView>

      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Step 2: Explore</ThemedText>
        <ThemedText>
          Tap the Explore tab to learn more about what's included in this starter app.
        </ThemedText>
      </ThemedView>

      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Step 3: Get a fresh start</ThemedText>
        <ThemedText>
          When you're ready, run{' '}
          <ThemedText type="defaultSemiBold">npm run reset-project</ThemedText> to get a fresh{' '}
          <ThemedText type="defaultSemiBold">app</ThemedText> directory.
        </ThemedText>
      </ThemedView>
    </ParallaxScrollView>
  );
}

const styles = StyleSheet.create({
  titleContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  stepContainer: {
    gap: 8,
    marginBottom: 8,
  },
  reactLogo: {
    height: 178,
    width: 290,
    bottom: 0,
    left: 0,
    position: 'absolute',
  },
});
