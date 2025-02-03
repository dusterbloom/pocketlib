import { Image, StyleSheet, Platform, View, TextInput } from 'react-native';
import { HelloWave } from '@/components/HelloWave';
import ParallaxScrollView from '@/components/ParallaxScrollView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';

import {
  generateKeys,
  generateAddress,
  createNote,
  signNote,
  verifySignature,
  createIntentAction,
  type AddressData,
  type KeyPair,
  type Note,
  type SignedNote,
  type NoteCreateParams,
  type CreateIntentActionParams
} from '../../modules/proofmanager/index';

import { Text } from "react-native";
import { bytesToHex } from '@/utils/hex';

import { useEffect, useState } from 'react';
import React from 'react';

// Debug logging utility
const debug = (step: string, data: any) => {
  console.log(`\n[DEBUG] ${step}:`);
  console.log(JSON.stringify(data, null, 2));
};

export default function HomeScreen() {
  // Updated state
  const [debtorAddress, setDebtorAddress] = useState<AddressData | null>(null);
  const [creditorAddress, setCreditorAddress] = useState<AddressData | null>(null);
  const [note, setNote] = useState<Note | null>(null);
  const [signedNote, setSignedNote] = useState<SignedNote | null>(null);
  const [isValid, setIsValid] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [amount, setAmount] = useState<string>('1000');
  const [assetId, setAssetId] = useState<string>('1');

  const [timings, setTimings] = useState<{
    keyGenTime: number | null;
    addressGenTime: number | null;
    noteCreateTime: number | null;
    signTime: number | null;
    verifyTime: number | null;
    intentTime: number | null;  
  }>({
    keyGenTime: null,
    addressGenTime: null,
    noteCreateTime: null,
    signTime: null,
    verifyTime: null,
    intentTime: null
  });

  useEffect(() => {
    async function generateAndVerify() {
      try {
        console.log('\n=== Starting ProofManager Flow ===');

        // Test seed phrases
        const debtorPhrase = 'garage advice weekend this dose mango sign horse tool torch mosquito repeat sentence valid scheme pull punch need prosper build actor say cancel allow';
        const creditorPhrase = 'word word word word word word word word word word word word';

        // debug('Seed Phrases', { debtorPhrase, creditorPhrase });

        // Generate keys for debtor
        console.log('\n[Step 1] Generating Debtor Keys...');
        const startKeyGen = Date.now();
        const debtorKeys = await generateKeys(debtorPhrase);
        const endKeyGen = Date.now();
        debug('Debtor Keys Generated', debtorKeys);
        setTimings(prev => ({ ...prev, keyGenTime: endKeyGen - startKeyGen }));

        // Generate addresses
        console.log('\n[Step 2] Generating Addresses...');
        const startAddr = Date.now();

        // debug('Calling generateAddress with', {
        //   spendKey: debtorKeys.spendKey,
        //   index: 0
        // });

        const debtorAddr = await generateAddress({
          spendKey: debtorKeys.spendKey,
          index: 0
        });
        // debug('Debtor Address Generated', debtorAddr);

        const creditorKeys = await generateKeys(creditorPhrase);
        // debug('Creditor Keys Generated', creditorKeys);

        const creditorAddr = await generateAddress({
          spendKey: creditorKeys.spendKey,
          index: 0
        });
        // debug('Creditor Address Generated', creditorAddr);

        const endAddr = Date.now();

        setDebtorAddress(debtorAddr);
        setCreditorAddress(creditorAddr);
        setTimings(prev => ({ ...prev, addressGenTime: endAddr - startAddr }));

        // Create note
        console.log('\n[Step 3] Creating Note...');
        const startNote = Date.now();

        const noteParams = {
          debtorAddress: debtorAddr,
          creditorAddress: creditorAddr,
          amount: parseInt(amount),
          assetId: parseInt(assetId)
        };
        // debug('Creating Note with params', noteParams);

        const newNote = await createNote(noteParams);
        debug('Note Created', newNote);

        const endNote = Date.now();
        setNote(newNote);
        setTimings(prev => ({ ...prev, noteCreateTime: endNote - startNote }));

        // Sign note
        console.log('\n[Step 4] Signing Note...');
        const startSign = Date.now();
        debug('Signing note with params', {
          seedPhrase: debtorPhrase,
          note: newNote
        });

        const signed = await signNote({
          seedPhrase: debtorPhrase,
          note: newNote
        });
        debug('Note Signed', signed);

        const endSign = Date.now();
        setSignedNote(signed);
        setTimings(prev => ({ ...prev, signTime: endSign - startSign }));

        console.log('\n[Step 5] Verifying Signature...');
        const startVerify = Date.now();

        if (!signed.verificationKey || !signed.signature || !signed.note?.commitment) {
          throw new Error('Missing required signature components');
        }

        // Debug logging
        console.log('Verifying with:', {
          verificationKey: bytesToHex(signed.verificationKey),
          commitment: signed.note.commitment ? bytesToHex(signed.note.commitment) : 'missing',
          signature: bytesToHex(signed.signature)
        });

        try {
          const valid = await verifySignature({
            verificationKey: signed.verificationKey,
            commitment: signed.note.commitment,
            signature: signed.signature
          });

          console.log('Verification result:', valid);
          setIsValid(valid);
          const endVerify = Date.now();
          setTimings(prev => ({ ...prev, verifyTime: endVerify - startVerify }));
        } catch (err) {
          console.error('Verification error:', err);
          setError(`Verification failed: ${err.message}`);
        }

        // Add after verifying signature in useEffect
        console.log('\n[Step 6] Creating Intent Action...');
        const startIntent = Date.now();

      // Debug logging
      console.log('CreateIntent with:', {
        debtorSeedPhrase: Array.from(new TextEncoder().encode(debtorPhrase)),
        rseedRandomness: Array(32).fill(0).map(() => Math.floor(Math.random() * 256)),
        debtorIndex: 0,
        creditorAddr: "penumbra1qvsnxx2hq7ekuxavw3k35fgzwtfa0na9n7kquz5gvzzly05rpjsyma0rdyqctkyxe75sh3afdhsj033zne6en332aaglrj4fk7vge7apaepjlz4a07fphk5l206vrhsphfk2da", // Or appropriate address format
        amount: parseInt(amount),
        assetId: parseInt(assetId)
      });


        try {
          const intentResult = await createIntentAction({
            debtorSeedPhrase: Array.from(new TextEncoder().encode(debtorPhrase)),
            rseedRandomness: Array(32).fill(0).map(() => Math.floor(Math.random() * 256)),
            debtorIndex: 0,
            creditorAddr: "penumbra1qvsnxx2hq7ekuxavw3k35fgzwtfa0na9n7kquz5gvzzly05rpjsyma0rdyqctkyxe75sh3afdhsj033zne6en332aaglrj4fk7vge7apaepjlz4a07fphk5l206vrhsphfk2da", // Or appropriate address format
            amount: parseInt(amount),
            assetId: parseInt(assetId)
          });

          debug('Intent Action Created', intentResult);
          const endIntent = Date.now();
          setTimings(prev => ({ ...prev, intentTime: endIntent - startIntent }));
        } catch (err) {
          console.error('Intent creation error:', err);
          setError(`Intent creation failed: ${err.message}`);
        }



        console.log('\n=== ProofManager Flow Completed ===\n');

      } catch (err) {
        console.error('\n[ERROR] ProofManager Flow Failed:', err);
        console.error('Error Stack:', err instanceof Error ? err.stack : 'No stack trace');
        setError(err instanceof Error ? err.message : 'Unknown error');
      }
    }

    generateAndVerify();
  }, [amount, assetId]);

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
        <ThemedText type="subtitle">Note Parameters</ThemedText>
        <View style={styles.inputContainer}>
          <Text>Amount: </Text>
          <TextInput
            value={amount}
            onChangeText={setAmount}
            keyboardType="numeric"
            style={styles.input}
          />
        </View>
        <View style={styles.inputContainer}>
          <Text>Asset ID: </Text>
          <TextInput
            value={assetId}
            onChangeText={setAssetId}
            keyboardType="numeric"
            style={styles.input}
          />
        </View>
      </ThemedView>
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Note Creation Results</ThemedText>
        {note && (
          <>
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
        {timings.intentTime != null && (
          <Text>Intent Creation: {timings.intentTime} ms</Text>
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