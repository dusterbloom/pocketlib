import { Image, StyleSheet, Platform } from 'react-native';

import { HelloWave } from '@/components/HelloWave';
import ParallaxScrollView from '@/components/ParallaxScrollView';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';

import * as ProofManager from '@/modules/proofmanager'
import { ProofInput, SerializedProof } from '../../modules/proofmanager/src/ProofManager.types';
import { useEffect, useState } from 'react';
import { Text, View } from "react-native";




export default function HomeScreen() {
  const [proofResult, setProofResult] = useState<SerializedProof | null>(null);
  const [error, setError] = useState<string | null>(null);
  // const [value, setValue] = useState<null | any>(null);
  
  

  useEffect(() => {
    async function generateProof() {
        try {
            const input: ProofInput = {
                seedPhrase: "garage advice weekend this dose mango sign horse tool torch mosquito repeat sentence valid scheme pull punch need prosper build actor say cancel allow",
                amount: 1000,
                assetId: 1,
                addressIndex: 0
            };

            console.log("Generating proof with input:", input);
            
            // First generate an address - we'll need this
            const address = await ProofManager.generateAddress(input.seedPhrase, input.addressIndex);
            console.log("Generated address:", address);

            // Generate proof
            const proof = await ProofManager.createProof(input);
            console.log("Generated proof:", proof);
            setProofResult(proof);

            // The commitment should be derived from the note created with our proof
            // For testing, you can print out the actual commitment from your Rust code
            // by adding debug prints in the createProof function
            
            // Generate proof and get commitment
            const result = await ProofManager.createProof(input);
            console.log("Generated proof result:", {
              proofLength: result.proof.length,
              commitmentLength: result.commitment.length,
              proof: result.proof,
              commitment: result.commitment
            });

            // Now verify using the actual commitment
            const isValid = await ProofManager.verifyProof(result.proof, result.commitment);
            console.log("Proof verification result:", isValid);

        } catch (err) {
            console.error("Error in proof generation:", err);
            setError(err instanceof Error ? err.message : "Unknown error");
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
      }>
      <ThemedView style={styles.titleContainer}>
        {/* <ThemedText type="title">{hello()}!</ThemedText> */}
        <HelloWave />
      </ThemedView>
      <ThemedView style={styles.stepContainer}>
        <ThemedText type="subtitle">Step 1: Try it</ThemedText>
        <Text >
        { `The proof is: ${proofResult}`}
      </Text>


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
          <ThemedText type="defaultSemiBold">app</ThemedText> directory. This will move the current{' '}
          <ThemedText type="defaultSemiBold">app</ThemedText> to{' '}
          <ThemedText type="defaultSemiBold">app-example</ThemedText>.
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
