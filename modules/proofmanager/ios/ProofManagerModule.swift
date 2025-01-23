// modules/proofmanager/ios/ProofManagerModule.swift

import ExpoModulesCore

public class ProofManagerModule: Module {
    private let proofManager = try! ProofManager()
    
    public func definition() -> ModuleDefinition {
        Name("ProofManager")

        AsyncFunction("createProof") { (input: [String: Any]) -> [String: [Int]] in
            guard let seedPhrase = input["seedPhrase"] as? String,
                  let amount = input["amount"] as? NSNumber,
                  let assetId = input["assetId"] as? NSNumber,
                  let addressIndex = input["addressIndex"] as? NSNumber else {
                throw ProofError.InvalidSeed
            }

            let proofInput = ProofInput(
                seedPhrase: seedPhrase,
                amount: amount.uint64Value,
                assetId: assetId.uint64Value,
                addressIndex: UInt32(addressIndex.int32Value)
            )

            let result = try proofManager.createProofWithCommitment(input: proofInput)
            return [
                "proof": Array(result.proof.data),
                "commitment": Array(result.commitment)
            ]
        }

        AsyncFunction("verifyProof") { (proof: [Int], commitment: [Int]) -> Bool in
            let proofData = Data(proof.map { UInt8($0) })
            let commitmentData = Data(commitment.map { UInt8($0) })
            
            return try proofManager.verifyProof(
                proof: SerializedProof(data: proofData),
                commitment: commitmentData
            )
        }

        AsyncFunction("generateAddress") { (seedPhrase: String, index: Int) -> [String: [Int]] in
            let addressInfo = try proofManager.generateAddress(
                seedPhrase: seedPhrase,
                index: UInt32(index)
            )
            
            return [
                "diversifier": Array(addressInfo.diversifier),
                "transmissionKey": Array(addressInfo.transmissionKey),
                "clueKey": Array(addressInfo.clueKey)
            ]
        }
    }
}