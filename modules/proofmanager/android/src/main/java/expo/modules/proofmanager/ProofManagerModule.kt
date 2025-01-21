package expo.modules.proofmanager

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import java.net.URL

class ProofManagerModule : Module() {
    companion object {
        init {
            // Load the native Rust library
            try {
                System.loadLibrary("native_rust_lib")
            } catch (e: UnsatisfiedLinkError) {
                println("Failed to load native_rust_lib: ${e.message}")
            }
        }
    }

    // Native function declarations
    external fun rustAdd(a: Int, b: Int): Int
    external fun generateAddress(seedPhrase: String, index: Int): AddressInfo
    external fun createProof(input: ProofInput): ByteArray
    external fun verifyProof(proofData: ByteArray, commitment: ByteArray): Boolean

    // Data classes for proof operations
    data class AddressInfo(
        val diversifier: ByteArray,
        val transmissionKey: ByteArray,
        val clueKey: ByteArray
    )

    data class ProofInput(
        val seedPhrase: String,
        val amount: Long,
        val assetId: Long,
        val addressIndex: Int
    )

    override fun definition() = ModuleDefinition {
        Name("ProofManager")

        Constants(
            "PI" to Math.PI
        )

        Events("onChange", "onProofGenerated", "onError")

        // Basic test function
        Function("hello") {
            "Hello from ProofManager! 👋"
        }

        // Expose Rust functions
        AsyncFunction("rustAdd") { a: Int, b: Int ->
            try {
                rustAdd(a, b)
            } catch (e: Exception) {
                throw Error("Failed to execute rustAdd: ${e.message}")
            }
        }

        // Proof generation functions
        AsyncFunction("generateAddress") { seedPhrase: String, index: Int ->
            try {
                val address = generateAddress(seedPhrase, index)
                mapOf(
                    "diversifier" to address.diversifier,
                    "transmissionKey" to address.transmissionKey,
                    "clueKey" to address.clueKey
                )
            } catch (e: Exception) {
                throw Error("Failed to generate address: ${e.message}")
            }
        }

        AsyncFunction("createProof") { input: Map<String, Any> ->
            try {
                val proofInput = ProofInput(
                    seedPhrase = input["seedPhrase"] as String,
                    amount = (input["amount"] as Number).toLong(),
                    assetId = (input["assetId"] as Number).toLong(),
                    addressIndex = (input["addressIndex"] as Number).toInt()
                )
                val proofData = createProof(proofInput)
                // Send success event
                sendEvent("onProofGenerated", mapOf("proof" to proofData))
                mapOf("proof" to proofData)
            } catch (e: Exception) {
                sendEvent("onError", mapOf("error" to e.message))
                throw Error("Failed to create proof: ${e.message}")
            }
        }

        AsyncFunction("verifyProof") { proof: ByteArray, commitment: ByteArray ->
            try {
                verifyProof(proof, commitment)
            } catch (e: Exception) {
                throw Error("Failed to verify proof: ${e.message}")
            }
        }

        // View functionality for WebView if needed
        View(ProofManagerView::class) {
            Prop("url") { view: ProofManagerView, url: URL ->
                view.webView.loadUrl(url.toString())
            }
            Events("onLoad")
        }
    }
}