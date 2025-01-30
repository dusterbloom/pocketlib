import ExpoModulesCore
import Foundation
import proofmanager

public class ProofManagerModule: Module {
    // Initialize ProofManager from the generated bindings
    private lazy var proofManager: ProofManager = {
        do {
            print("Swift: Initializing ProofManager")
            return try ProofManager()
        } catch {
            print("Swift: Failed to initialize ProofManager: \(error)")
            fatalError("Failed to initialize ProofManager: \(error)")
        }
    }()
    
    public func definition() -> ModuleDefinition {
        Name("ProofManager")
        
        AsyncFunction("generateKeys") { (seedPhrase: String, promise: Promise) in
            do {
                print("Swift: Generating keys for seedPhrase")
                let keyPair = try proofManager.generateKeys(seedPhrase: seedPhrase)
                promise.resolve([
                    "spendKey": Array(keyPair.spendKey).map { Int($0) },
                    "viewKey": Array(keyPair.viewKey).map { Int($0) }
                ])
                print("Swift: Generated keys successfully")
            } catch {
                print("Swift: Error in generateKeys: \(error)")
                promise.reject("PROOF_ERROR", error.localizedDescription)
            }
        }
        
        AsyncFunction("generateAddress") { (spendKeyBytes: [Int], index: Int, promise: Promise) in
            do {
                print("Swift: Generating address for index=\(index)")
                let address = try proofManager.generateAddress(
                    spendKeyBytes: Data(spendKeyBytes.map { UInt8($0) }),
                    index: UInt32(index)
                )
                
                let result = [
                    "diversifier": Array(address.diversifier).map { Int($0) },
                    "transmissionKey": Array(address.transmissionKey).map { Int($0) },
                    "clueKey": Array(address.clueKey).map { Int($0) }
                ]
                promise.resolve(result)
                print("Swift: Generated address successfully")
            } catch {
                print("Swift: Error in generateAddress: \(error)")
                promise.reject("PROOF_ERROR", error.localizedDescription)
            }
        }
        
        AsyncFunction("createNote") { (args: [String: Any], promise: Promise) in
            do {
                print("Swift: Creating note")
                guard let debtorMap = args["debtorAddress"] as? [String: [Int]],
                      let creditorMap = args["creditorAddress"] as? [String: [Int]],
                      let amount = args["amount"] as? NSNumber,
                      let assetId = args["assetId"] as? NSNumber else {
                    throw ProofError.invalidKey
                }
                
                let debtorAddress = AddressData(
                    diversifier: Data(debtorMap["diversifier"]!.map { UInt8($0) }),
                    transmissionKey: Data(debtorMap["transmissionKey"]!.map { UInt8($0) }),
                    clueKey: Data(debtorMap["clueKey"]!.map { UInt8($0) })
                )
                
                let creditorAddress = AddressData(
                    diversifier: Data(creditorMap["diversifier"]!.map { UInt8($0) }),
                    transmissionKey: Data(creditorMap["transmissionKey"]!.map { UInt8($0) }),
                    clueKey: Data(creditorMap["clueKey"]!.map { UInt8($0) })
                )
                
                let note = try proofManager.createNote(
                    debtorAddress: debtorAddress,
                    creditorAddress: creditorAddress,
                    amount: UInt64(truncating: amount),
                    assetId: UInt64(truncating: assetId)
                )
                
                let result: [String: Any] = [
                    "debtorAddress": [
                        "diversifier": Array(note.debtorAddress.diversifier).map { Int($0) },
                        "transmissionKey": Array(note.debtorAddress.transmissionKey).map { Int($0) },
                        "clueKey": Array(note.debtorAddress.clueKey).map { Int($0) }
                    ],
                    "creditorAddress": [
                        "diversifier": Array(note.creditorAddress.diversifier).map { Int($0) },
                        "transmissionKey": Array(note.creditorAddress.transmissionKey).map { Int($0) },
                        "clueKey": Array(note.creditorAddress.clueKey).map { Int($0) }
                    ],
                    "amount": note.amount,
                    "assetId": note.assetId,
                    "commitment": Array(note.commitment).map { Int($0) }
                ]
                promise.resolve(result)
                print("Swift: Created note successfully")
            } catch {
                print("Swift: Error in createNote: \(error)")
                promise.reject("PROOF_ERROR", error.localizedDescription)
            }
        }
        
        AsyncFunction("signNote") { (seedPhrase: String, noteMap: [String: Any], promise: Promise) in
            do {
                print("Swift: Signing note")
                guard let debtorMap = noteMap["debtorAddress"] as? [String: [Int]],
                      let creditorMap = noteMap["creditorAddress"] as? [String: [Int]],
                      let amount = noteMap["amount"] as? NSNumber,
                      let assetId = noteMap["assetId"] as? NSNumber,
                      let commitment = noteMap["commitment"] as? [Int] else {
                    throw ProofError.invalidKey
                }
                
                let note = Note(
                    debtorAddress: AddressData(
                        diversifier: Data(debtorMap["diversifier"]!.map { UInt8($0) }),
                        transmissionKey: Data(debtorMap["transmissionKey"]!.map { UInt8($0) }),
                        clueKey: Data(debtorMap["clueKey"]!.map { UInt8($0) })
                    ),
                    creditorAddress: AddressData(
                        diversifier: Data(creditorMap["diversifier"]!.map { UInt8($0) }),
                        transmissionKey: Data(creditorMap["transmissionKey"]!.map { UInt8($0) }),
                        clueKey: Data(creditorMap["clueKey"]!.map { UInt8($0) })
                    ),
                    amount: UInt64(truncating: amount),
                    assetId: UInt64(truncating: assetId),
                    commitment: Data(commitment.map { UInt8($0) })
                )
                
                let signedNote = try proofManager.signNote(seedPhrase: seedPhrase, note: note)
                let result: [String: Any] = [
                    "note": [
                        "debtorAddress": [
                            "diversifier": Array(signedNote.note.debtorAddress.diversifier).map { Int($0) },
                            "transmissionKey": Array(signedNote.note.debtorAddress.transmissionKey).map { Int($0) },
                            "clueKey": Array(signedNote.note.debtorAddress.clueKey).map { Int($0) }
                        ],
                        "creditorAddress": [
                            "diversifier": Array(signedNote.note.creditorAddress.diversifier).map { Int($0) },
                            "transmissionKey": Array(signedNote.note.creditorAddress.transmissionKey).map { Int($0) },
                            "clueKey": Array(signedNote.note.creditorAddress.clueKey).map { Int($0) }
                        ],
                        "amount": signedNote.note.amount,
                        "assetId": signedNote.note.assetId,
                        "commitment": Array(signedNote.note.commitment).map { Int($0) }
                    ],
                    "signature": Array(signedNote.signature).map { Int($0) },
                    "verificationKey": Array(signedNote.verificationKey).map { Int($0) }
                ]
                promise.resolve(result)
                print("Swift: Signed note successfully")
            } catch {
                print("Swift: Error in signNote: \(error)")
                promise.reject("PROOF_ERROR", error.localizedDescription)
            }
        }
        
        AsyncFunction("verifySignature") { (verificationKey: [Int], commitment: [Int], signature: [Int], promise: Promise) in
            do {
                print("Swift: Verifying signature")
                let result = try proofManager.verifySignature(
                    verificationKeyBytes: Data(verificationKey.map { UInt8($0) }),
                    commitment: Data(commitment.map { UInt8($0) }),
                    signature: Data(signature.map { UInt8($0) })
                )
                promise.resolve(result)
                print("Swift: Verified signature successfully: \(result)")
            } catch {
                print("Swift: Error in verifySignature: \(error)")
                promise.reject("PROOF_ERROR", error.localizedDescription)
            }
        }
    }
}