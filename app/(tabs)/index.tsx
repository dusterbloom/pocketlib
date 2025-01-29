import { Image, StyleSheet, Platform } from 'react-native';
import { HelloWave } from '@/components/HelloWave';
import ParallaxScrollView from '@/components/ParallaxScrollView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';

import * as ProofManager from '@/modules/proofmanager'
import { ProofInput, IntentAction } from '@/modules/proofmanager/src/ProofManager.types';
import { useEffect, useState } from 'react';
import { Text } from "react-native";
import { addressToHex, bytesToHex, convertByteArraysToHex } from '@/utils/hex';

export default function HomeScreen() {
  // Existing state
  const [addressHex, setAddressHex] = useState<{
    clueKey: string;
    diversifier: string;
    transmissionKey: string;
  } | null>(null);

  const [proofHex, setProofHex] = useState<string | null>(null);
  const [commitmentHex, setCommitmentHex] = useState<string | null>(null);
  const [isValid, setIsValid] = useState<boolean | null>(null);

  // New state for intent actions
  const [intentAction, setIntentAction] = useState<IntentAction | null>(null);
  const [intentIsValid, setIntentIsValid] = useState<boolean | null>(null);

  const [error, setError] = useState<string | null>(null);  
  const [timings, setTimings] = useState<{
    addressGenTime: number | null;
    proofGenTime: number | null;
    verifyTime: number | null;
    intentGenTime: number | null;
    intentVerifyTime: number | null;
  }>({
    addressGenTime: null,
    proofGenTime: null,
    verifyTime: null,
    intentGenTime: null,
    intentVerifyTime: null,
  });

  useEffect(() => {
    async function generateProof() {
      try {
        // Existing seed phrases
        const debtorPhrase = 'garage advice weekend this dose mango sign horse tool torch mosquito repeat sentence valid scheme pull punch need prosper build actor say cancel allow';
        const creditorPhrase = 'word word word word word word word word word word word word';

        const input: ProofInput = {
          seedPhrase: debtorPhrase,
          amount: 1000,
          assetId: 1,
          addressIndex: 0,
        };

        // --- Existing ZKP Generation Flow ---
        console.log('Generating proof with input:', input);

        const startAddr = Date.now();
        const address = await ProofManager.generateAddress(
          input.seedPhrase,
          input.addressIndex
        );
        const endAddr = Date.now();
        const addressGenTime = endAddr - startAddr;

        const hexAddr = convertByteArraysToHex(address);
        setAddressHex(hexAddr);
        setTimings((prev) => ({ ...prev, addressGenTime }));

        const startProof = Date.now();
        const result = await ProofManager.createProof(input);
        const endProof = Date.now();
        const proofGenTime = endProof - startProof;

        const proofHexString = convertByteArraysToHex(result.proof);
        const commitmentHexString = convertByteArraysToHex(result.commitment);
        setProofHex(proofHexString);
        setCommitmentHex(commitmentHexString);
        setTimings((prev) => ({ ...prev, proofGenTime }));

        const startVerify = Date.now();
        const valid = await ProofManager.verifyProof(
          result.proof,
          result.commitment
        );
        const endVerify = Date.now();
        setIsValid(valid);
        setTimings((prev) => ({ ...prev, verifyTime: endVerify - startVerify }));

        // --- Intent Action Flow ---
console.log('Generating intent action...');

// First get creditor's address (in a real app this would be passed in)
const creditorAddress = await ProofManager.generateAddress(
  'test test test test test test test test test test test junk',
  1  // Different index for creditor
);

        const startIntent = Date.now();
        const intentResult = await ProofManager.createIntentAction(
          debtorPhrase,
          30, // Amount
          1,  // TEST_ASSET_ID
          1,  // address index
          creditorAddress
        );
        const endIntent = Date.now();
        setIntentAction(intentResult);
        setTimings((prev) => ({ ...prev, intentGenTime: endIntent - startIntent }));

        const startIntentVerify = Date.now();
        const intentValid = await ProofManager.verifyIntentAction(intentResult);
        const endIntentVerify = Date.now();
        setIntentIsValid(intentValid);
        setTimings((prev) => ({ 
          ...prev, 
          intentVerifyTime: endIntentVerify - startIntentVerify 
        }));

      } catch (err) {
        console.error('Error:', err);
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

      {/* ZKP Section */}
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">ZKP Generation Results</ThemedText>

        <Text>Proof (hex): {proofHex ?? 'Generating...'}</Text>
        <Text>Commitment (hex): {commitmentHex ?? 'Generating...'}</Text>

        {addressHex && (
          <>
            <Text>Address ClueKey (hex): {addressHex.clueKey}</Text>
            <Text>Address Diversifier (hex): {addressHex.diversifier}</Text>
            <Text>Address TransmissionKey (hex): {addressHex.transmissionKey}</Text>
          </>
        )}

        {isValid !== null && (
          <Text>Proof verification: {isValid ? 'Valid ✅' : 'Invalid ❌'}</Text>
        )}
      </ThemedView>

      {/* Intent Action Section */}
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Intent Action Results</ThemedText>

        {intentAction && (
          <>
            <Text>Note Commitment (hex): {convertByteArraysToHex(intentAction.noteCommitment)}</Text>
            <Text>Auth Signature (hex): {convertByteArraysToHex(intentAction.authSig)}</Text>
            <Text>Verification Key (hex): {convertByteArraysToHex(intentAction.rk)}</Text>
          </>
        )}

        {intentIsValid !== null && (
          <Text>Intent verification: {intentIsValid ? 'Valid ✅' : 'Invalid ❌'}</Text>
        )}
      </ThemedView>

      {/* Error Display */}
      {error && (
        <ThemedView style={styles.stepContainer}>
          <Text style={{ color: 'red' }}>Error: {error}</Text>
        </ThemedView>
      )}

      {/* Timing Results */}
      <ThemedView style={styles.stepContainer}>
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
        {timings.intentGenTime != null && (
          <Text>Intent Generation: {timings.intentGenTime} ms</Text>
        )}
        {timings.intentVerifyTime != null && (
          <Text>Intent Verification: {timings.intentVerifyTime} ms</Text>
        )}
      </ThemedView>

      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Developer Tools</ThemedText>
        <ThemedText>
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