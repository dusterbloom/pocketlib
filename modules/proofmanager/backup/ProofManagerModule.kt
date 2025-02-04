package expo.modules.proofmanager

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

class ProofManagerModule : Module() {
    companion object {
        init {
            try {
                // Try both possible library names
                try {
                    System.loadLibrary("proofmanager")
                } catch (e: UnsatisfiedLinkError) {
                    System.loadLibrary("native_rust_lib")
                }
                println("Successfully loaded native library")
            } catch (e: UnsatisfiedLinkError) {
                println("Failed to load native library: ${e.message}")
                e.printStackTrace()
            }
        }
    }

    // Native function declarations - need to be at class level
    private external fun createProofNative(
        seedPhrase: String,
        amount: Long,
        assetId: Long,
        addressIndex: Int
    ): Map<String, ByteArray>


    private external fun verifyProofNative(proof: ByteArray, commitment: ByteArray): Boolean

    private external fun generateAddressNative(seedPhrase: String, index: Int): Map<String, ByteArray>

    override fun definition() = ModuleDefinition {
        Name("ProofManager")

        AsyncFunction("createProof") { input: Map<String, Any> ->
        try {
            val result = createProofNative(
                seedPhrase = input["seedPhrase"] as String,
                amount = (input["amount"] as Number).toLong(),
                assetId = (input["assetId"] as Number).toLong(),
                addressIndex = (input["addressIndex"] as Number).toInt()
            )
        
            mapOf(
                "proof" to result["proof"]?.map { it.toInt() and 0xFF },
                "commitment" to result["commitment"]?.map { it.toInt() and 0xFF }
            )
            } catch (e: Exception) {
                throw Error("Failed to create proof: ${e.message}")
            }
        }

        AsyncFunction("verifyProof") { proof: List<Int>, commitment: List<Int> ->
            try {
                println("Kotlin: Verifying proof of length=${proof.size}")
                
                // Add null checks and validation
                if (proof.isEmpty()) {
                    throw Error("Proof data cannot be empty")
                }
                if (commitment.isEmpty()) {
                    throw Error("Commitment data cannot be empty")
                }
                
                // Safely convert the lists to byte arrays with null checking
                val proofBytes = proof.mapNotNull { value -> 
                    value?.toByte()
                }.toByteArray()
                
                val commitmentBytes = commitment.mapNotNull { value ->
                    value?.toByte()
                }.toByteArray()
                
                println("Kotlin: Converting proof (${proofBytes.size} bytes) and commitment (${commitmentBytes.size} bytes)")
                
                val result = verifyProofNative(proofBytes, commitmentBytes)
                println("Kotlin: Verification result: $result")
                result
            } catch (e: Exception) {
                println("Kotlin: Error in verifyProof: ${e.message}")
                e.printStackTrace()
                throw Error("Failed to verify proof: ${e.message}")
            }
        }

        AsyncFunction("generateAddress") { seedPhrase: String, index: Int ->
            try {
                println("Kotlin: Generating address for index=$index")
                val addressInfo = generateAddressNative(seedPhrase, index)
                mapOf(
                    "diversifier" to addressInfo["diversifier"]?.map { it.toInt() and 0xFF },
                    "transmissionKey" to addressInfo["transmissionKey"]?.map { it.toInt() and 0xFF },
                    "clueKey" to addressInfo["clueKey"]?.map { it.toInt() and 0xFF }
                ).also { println("Kotlin: Generated address info: $it") }
            } catch (e: Exception) {
                println("Kotlin: Error in generateAddress: ${e.message}")
                e.printStackTrace()
                throw Error("Failed to generate address: ${e.message}")
            }
        }
    }
}