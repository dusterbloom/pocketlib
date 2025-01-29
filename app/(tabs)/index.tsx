import { Image, StyleSheet, Platform } from 'react-native';
import { HelloWave } from '@/components/HelloWave';
import ParallaxScrollView from '@/components/ParallaxScrollView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';

import * as ProofManager from '@/modules/proofmanager'
import { useEffect, useState } from 'react';
import { Text } from "react-native";
import { addressToHex, bytesToHex, convertByteArraysToHex } from '@/utils/hex';

import { AddressData, KeyPair, Note, SignedNote } from '@/modules/proofmanager/src/ProofManager.types';

export default function HomeScreen() {
  // Updated state
  const [debtorAddress, setDebtorAddress] = useState<AddressData | null>(null);
  const [creditorAddress, setCreditorAddress] = useState<AddressData | null>(null);
  const [note, setNote] = useState<Note | null>(null);
  const [signedNote, setSignedNote] = useState<SignedNote | null>(null);
  const [isValid, setIsValid] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  const [timings, setTimings] = useState<{
    keyGenTime: number | null;
    addressGenTime: number | null;
    noteCreateTime: number | null;
    signTime: number | null;
    verifyTime: number | null;
  }>({
    keyGenTime: null,
    addressGenTime: null,
    noteCreateTime: null,
    signTime: null,
    verifyTime: null,
  });

  useEffect(() => {
    async function generateAndVerify() {
      try {
        // Test seed phrases
        const debtorPhrase = 'garage advice weekend this dose mango sign horse tool torch mosquito repeat sentence valid scheme pull punch need prosper build actor say cancel allow';
        const creditorPhrase = 'word word word word word word word word word word word word';

        // Generate keys for debtor
        const startKeyGen = Date.now();
        const debtorKeys = await ProofManager.generateKeys(debtorPhrase);
        const endKeyGen = Date.now();
        setTimings(prev => ({ ...prev, keyGenTime: endKeyGen - startKeyGen }));

        // Generate addresses
        const startAddr = Date.now();
        const debtorAddr = await ProofManager.generateAddress(debtorKeys, 0);
        const creditorKeys = await ProofManager.generateKeys(creditorPhrase);
        const creditorAddr = await ProofManager.generateAddress(creditorKeys, 0);
        const endAddr = Date.now();
        
        setDebtorAddress(debtorAddr);
        setCreditorAddress(creditorAddr);
        setTimings(prev => ({ ...prev, addressGenTime: endAddr - startAddr }));

        // Create note
        const startNote = Date.now();
        const newNote = await ProofManager.createNote(
          debtorAddr,
          creditorAddr,
          1000, // amount
          1     // asset_id
        );
        const endNote = Date.now();
        setNote(newNote);
        setTimings(prev => ({ ...prev, noteCreateTime: endNote - startNote }));

        // Sign note
        const startSign = Date.now();
        const signed = await ProofManager.signNote(debtorPhrase, newNote);
        const endSign = Date.now();
        setSignedNote(signed);
        setTimings(prev => ({ ...prev, signTime: endSign - startSign }));

        // Verify signature
        const startVerify = Date.now();
        const valid = await ProofManager.verifySignature(
          signed.verificationKey,
          signed.note.commitment,
          signed.signature
        );
        const endVerify = Date.now();
        setIsValid(valid);
        setTimings(prev => ({ ...prev, verifyTime: endVerify - startVerify }));

      } catch (err) {
        console.error('Error:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
      }
    }

    generateAndVerify();
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

      {/* Address Section */}
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Address Generation Results</ThemedText>
        {debtorAddress && (
          <>
            <Text>Debtor Address:</Text>
            <Text>Diversifier: {bytesToHex(debtorAddress.diversifier)}</Text>
            <Text>Transmission Key: {bytesToHex(debtorAddress.transmissionKey)}</Text>
            <Text>Clue Key: {bytesToHex(debtorAddress.clueKey)}</Text>
          </>
        )}
      </ThemedView>

      {/* Note Section */}
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Note Creation Results</ThemedText>
        {note && (
          <>
            <Text>Amount: {note.amount}</Text>
            <Text>Asset ID: {note.assetId}</Text>
            <Text>Commitment: {bytesToHex(note.commitment)}</Text>
          </>
        )}
      </ThemedView>

      {/* Signature Section */}
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Signature Results</ThemedText>
        {signedNote && (
          <>
            <Text>Signature: {bytesToHex(signedNote.signature)}</Text>
            <Text>Verification Key: {bytesToHex(signedNote.verificationKey)}</Text>
          </>
        )}
        {isValid !== null && (
          <Text>Signature verification: {isValid ? 'Valid ✅' : 'Invalid ❌'}</Text>
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
        {timings.keyGenTime != null && (
          <Text>Key Generation: {timings.keyGenTime} ms</Text>
        )}
        {timings.addressGenTime != null && (
          <Text>Address Generation: {timings.addressGenTime} ms</Text>
        )}
        {timings.noteCreateTime != null && (
          <Text>Note Creation: {timings.noteCreateTime} ms</Text>
        )}
        {timings.signTime != null && (
          <Text>Note Signing: {timings.signTime} ms</Text>
        )}
        {timings.verifyTime != null && (
          <Text>Signature Verification: {timings.verifyTime} ms</Text>
        )}
      </ThemedView>

      {/* Developer Tools Section */}
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