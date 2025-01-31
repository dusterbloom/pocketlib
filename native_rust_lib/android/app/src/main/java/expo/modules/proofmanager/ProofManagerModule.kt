package expo.modules.proofmanager

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import expo.modules.kotlin.Promise

class ProofManagerModule : Module() {
    
    // Adding our library
    companion object {
        init {
            try {
                System.loadLibrary("proofmanager")
                println("Successfully loaded native library")
            } catch (e: UnsatisfiedLinkError) {
                println("Failed to load native library: ${e.message}")
                e.printStackTrace()
            }
        }
    }

    // Native function declarations matching Rust implementations
    private external fun generateKeysNative(seedPhrase: String): Map<String, ByteArray>
    
    private external fun generateAddressNative(
        spendKey: ByteArray,
        index: Int
    ): Map<String, ByteArray>

    private external fun createNoteNative(
        debtorAddress: Map<String, ByteArray>,
        creditorAddress: Map<String, ByteArray>,
        amount: Long,
        assetId: Long
    ): Map<String, ByteArray>

    private external fun signNoteNative(
        seedPhrase: String,
        note: Map<String, ByteArray>
    ): Map<String, ByteArray>
    
    private external fun verifySignatureNative(
        verificationKey: ByteArray,
        commitment: ByteArray,
        signature: ByteArray
    ): Boolean

    override fun definition() = ModuleDefinition {
        Name("ProofManager")

        AsyncFunction("generateKeys") { seedPhrase: String, promise: Promise ->
            try {
                val result = generateKeysNative(seedPhrase)
                promise.resolve(mapOf(
                    "spendKey" to (result["spendKey"]?.map { it.toInt() and 0xFF } ?: listOf()),
                    "viewKey" to (result["viewKey"]?.map { it.toInt() and 0xFF } ?: listOf())
                ))
            } catch (e: Exception) {
                promise.reject("PROOF_ERROR", e.message, e)
            }
        }

        AsyncFunction("generateAddress") { args: Map<String, Any>, promise: Promise ->
            try {
                val spendKey = (args["spendKey"] as? List<Int>)?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid spendKey")
                val index = (args["index"] as? Number)?.toInt()
                    ?: throw IllegalArgumentException("Invalid index")

                val result = generateAddressNative(spendKey, index)
                promise.resolve(mapOf(
                    "diversifier" to (result["diversifier"]?.map { it.toInt() and 0xFF } ?: listOf()),
                    "transmissionKey" to (result["transmissionKey"]?.map { it.toInt() and 0xFF } ?: listOf()),
                    "clueKey" to (result["clueKey"]?.map { it.toInt() and 0xFF } ?: listOf())
                ))
            } catch (e: Exception) {
                promise.reject("PROOF_ERROR", e.message, e)
            }
        }

        AsyncFunction("createNote") { args: Map<String, Any>, promise: Promise ->
            try {
                fun convertAddress(address: Map<*, *>): Map<String, ByteArray> {
                    return mapOf(
                        "diversifier" to ((address["diversifier"] as? List<Int>)?.map { it.toByte() }?.toByteArray()
                            ?: throw IllegalArgumentException("Invalid diversifier")),
                        "transmissionKey" to ((address["transmissionKey"] as? List<Int>)?.map { it.toByte() }?.toByteArray()
                            ?: throw IllegalArgumentException("Invalid transmissionKey")),
                        "clueKey" to ((address["clueKey"] as? List<Int>)?.map { it.toByte() }?.toByteArray()
                            ?: throw IllegalArgumentException("Invalid clueKey"))
                    )
                }

                val debtorAddress = (args["debtorAddress"] as? Map<*, *>)?.let { convertAddress(it) }
                    ?: throw IllegalArgumentException("Invalid debtorAddress")
                val creditorAddress = (args["creditorAddress"] as? Map<*, *>)?.let { convertAddress(it) }
                    ?: throw IllegalArgumentException("Invalid creditorAddress")
                val amount = (args["amount"] as? Number)?.toLong()
                    ?: throw IllegalArgumentException("Invalid amount")
                val assetId = (args["assetId"] as? Number)?.toLong()
                    ?: throw IllegalArgumentException("Invalid assetId")

                val result = createNoteNative(debtorAddress, creditorAddress, amount, assetId)
                promise.resolve(mapOf(
                    "commitment" to (result["commitment"]?.map { it.toInt() and 0xFF } ?: listOf()),
                    "debtorAddress" to mapOf(
                        "diversifier" to (result["debtorDiversifier"]?.map { it.toInt() and 0xFF } ?: listOf()),
                        "transmissionKey" to (result["debtorTransmissionKey"]?.map { it.toInt() and 0xFF } ?: listOf()),
                        "clueKey" to (result["debtorClueKey"]?.map { it.toInt() and 0xFF } ?: listOf())
                    ),
                    "creditorAddress" to mapOf(
                        "diversifier" to (result["creditorDiversifier"]?.map { it.toInt() and 0xFF } ?: listOf()),
                        "transmissionKey" to (result["creditorTransmissionKey"]?.map { it.toInt() and 0xFF } ?: listOf()),
                        "clueKey" to (result["creditorClueKey"]?.map { it.toInt() and 0xFF } ?: listOf())
                    )
                ))
            } catch (e: Exception) {
                promise.reject("PROOF_ERROR", e.message, e)
            }
        }

        AsyncFunction("signNote") { args: Map<String, Any>, promise: Promise ->
            try {
                val seedPhrase = args["seedPhrase"] as? String
                    ?: throw IllegalArgumentException("Invalid seedPhrase")
                    
                val noteMap = args["note"] as? Map<String, Any>
                    ?: throw IllegalArgumentException("Invalid note structure")
                
                // Convert the input note to the format expected by native code
                val processedNote = mutableMapOf<String, ByteArray>()
                
                // Extract commitment
                val commitment = (noteMap["commitment"] as? List<Int>)?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid commitment in note")
                processedNote["commitment"] = commitment

                // Extract debtor address
                val debtorAddress = noteMap["debtorAddress"] as? Map<*, *>
                    ?: throw IllegalArgumentException("Invalid debtorAddress")
                
                processedNote["debtorDiversifier"] = (debtorAddress["diversifier"] as? List<Int>)
                    ?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid debtor diversifier")
                processedNote["debtorTransmissionKey"] = (debtorAddress["transmissionKey"] as? List<Int>)
                    ?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid debtor transmissionKey")
                processedNote["debtorClueKey"] = (debtorAddress["clueKey"] as? List<Int>)
                    ?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid debtor clueKey")

                // Extract creditor address
                val creditorAddress = noteMap["creditorAddress"] as? Map<*, *>
                    ?: throw IllegalArgumentException("Invalid creditorAddress")
                
                processedNote["creditorDiversifier"] = (creditorAddress["diversifier"] as? List<Int>)
                    ?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid creditor diversifier")
                processedNote["creditorTransmissionKey"] = (creditorAddress["transmissionKey"] as? List<Int>)
                    ?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid creditor transmissionKey")
                processedNote["creditorClueKey"] = (creditorAddress["clueKey"] as? List<Int>)
                    ?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid creditor clueKey")

                val result = signNoteNative(seedPhrase, processedNote)
                
                // Convert the native result back to the format expected by TypeScript
                promise.resolve(mapOf(
                    "signature" to (result["signature"]?.map { it.toInt() and 0xFF } ?: listOf()),
                    "verificationKey" to (result["verificationKey"]?.map { it.toInt() and 0xFF } ?: listOf()),
                    "note" to mapOf(
                        "commitment" to (result["noteCommitment"]?.map { it.toInt() and 0xFF } ?: listOf()),
                        "debtorAddress" to mapOf(
                            "diversifier" to (result["debtorDiversifier"]?.map { it.toInt() and 0xFF } ?: listOf()),
                            "transmissionKey" to (result["debtorTransmissionKey"]?.map { it.toInt() and 0xFF } ?: listOf()),
                            "clueKey" to (result["debtorClueKey"]?.map { it.toInt() and 0xFF } ?: listOf())
                        ),
                        "creditorAddress" to mapOf(
                            "diversifier" to (result["creditorDiversifier"]?.map { it.toInt() and 0xFF } ?: listOf()),
                            "transmissionKey" to (result["creditorTransmissionKey"]?.map { it.toInt() and 0xFF } ?: listOf()),
                            "clueKey" to (result["creditorClueKey"]?.map { it.toInt() and 0xFF } ?: listOf())
                        )
                    )
                ))
            } catch (e: Exception) {
                println("Error in signNote: ${e.message}")
                e.printStackTrace()
                promise.reject("PROOF_ERROR", "Sign note failed: ${e.message}", e)
            }
        }

        AsyncFunction("verifySignature") { args: Map<String, Any>, promise: Promise ->
            try {
                val verificationKey = (args["verificationKey"] as? List<Int>)?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid verificationKey")
                val commitment = (args["commitment"] as? List<Int>)?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid commitment")
                val signature = (args["signature"] as? List<Int>)?.map { it.toByte() }?.toByteArray()
                    ?: throw IllegalArgumentException("Invalid signature")

                val result = verifySignatureNative(verificationKey, commitment, signature)
                promise.resolve(result)
            } catch (e: Exception) {
                promise.reject("PROOF_ERROR", e.message, e)
            }
        }
    }
}