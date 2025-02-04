import ExpoModulesCore

public class ProofManagerModule: Module {


    // Initialize ProofManager instance
    private var proofManager: ProofManager?
    
    public func definition() -> ModuleDefinition {
        Name("ProofManager")
        
        OnCreate {
            do {
                self.proofManager = try ProofManager()
            } catch {
                print("Failed to initialize ProofManager: \(error)")
            }
        }
        
        AsyncFunction("generateKeys") { (seedPhrase: String, promise: Promise) in
            guard let pm = self.proofManager else {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "ProofManager not initialized"]))
                return
            }
            
            do {
                let result = try pm.generateKeys(seedPhrase: seedPhrase)
                promise.resolve([
                    "spendKey": Array(result.spendKey),
                    "viewKey": Array(result.viewKey)
                ])
            } catch {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: error.localizedDescription]))
            }
        }
        
        AsyncFunction("generateAddress") { (args: [String: Any], promise: Promise) in
            guard let pm = self.proofManager else {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "ProofManager not initialized"]))
                return
            }
            
            guard let spendKeyArray = args["spendKey"] as? [Int],
                  let index = args["index"] as? Int else {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid arguments"]))
                return
            }
            
            do {
                let spendKey = Data(spendKeyArray.map { UInt8($0) })
                let result = try pm.generateAddress(spendKeyBytes: spendKey, index: UInt32(index))
                
                promise.resolve([
                    "diversifier": Array(result.diversifier),
                    "transmissionKey": Array(result.transmissionKey),
                    "clueKey": Array(result.clueKey)
                ])
            } catch {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: error.localizedDescription]))
            }
        }
        
        AsyncFunction("createNote") { (args: [String: Any], promise: Promise) in
            guard let pm = self.proofManager else {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "ProofManager not initialized"]))
                return
            }
            
            do {
                // Convert addresses from JS format
                func convertAddress(_ addressDict: [String: Any]) throws -> AddressData {
                    guard let diversifier = addressDict["diversifier"] as? [Int],
                          let transmissionKey = addressDict["transmissionKey"] as? [Int],
                          let clueKey = addressDict["clueKey"] as? [Int] else {
                        throw NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid address format"])
                    }
                    
                    return AddressData(
                        diversifier: Data(diversifier.map { UInt8($0) }),
                        transmissionKey: Data(transmissionKey.map { UInt8($0) }),
                        clueKey: Data(clueKey.map { UInt8($0) })
                    )
                }
                
                guard let debtorAddressDict = args["debtorAddress"] as? [String: Any],
                      let creditorAddressDict = args["creditorAddress"] as? [String: Any],
                      let amount = args["amount"] as? Int,
                      let assetId = args["assetId"] as? Int else {
                    throw NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid arguments"])
                }
                
                let debtorAddress = try convertAddress(debtorAddressDict)
                let creditorAddress = try convertAddress(creditorAddressDict)
                
                let result = try pm.createNote(
                    debtorAddress: debtorAddress,
                    creditorAddress: creditorAddress,
                    amount: UInt64(amount),
                    assetId: UInt64(assetId)
                )
                
                promise.resolve([
                    "commitment": Array(result.commitment),
                    "debtorAddress": [
                        "diversifier": Array(result.debtorAddress.diversifier),
                        "transmissionKey": Array(result.debtorAddress.transmissionKey),
                        "clueKey": Array(result.debtorAddress.clueKey)
                    ],
                    "creditorAddress": [
                        "diversifier": Array(result.creditorAddress.diversifier),
                        "transmissionKey": Array(result.creditorAddress.transmissionKey),
                        "clueKey": Array(result.creditorAddress.clueKey)
                      ],
                        "amount": Int(amount),   // <== Added
                        "assetId": Int(assetId)   // <== Added
                    ])
            } catch {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: error.localizedDescription]))
            }
        }
        
        AsyncFunction("signNote") { (args: [String: Any], promise: Promise) in
            guard let pm = self.proofManager else {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "ProofManager not initialized"]))
                return
            }
            
            do {
                guard let seedPhrase = args["seedPhrase"] as? String,
                      let noteDict = args["note"] as? [String: Any] else {
                    throw NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid arguments"])
                }
                
                // Convert note from JS format
                func convertNote(_ noteDict: [String: Any]) throws -> Note {
                    guard let commitment = (noteDict["commitment"] as? [Int])?.map({ UInt8($0) }),
                          let debtorAddressDict = noteDict["debtorAddress"] as? [String: Any],
                          let creditorAddressDict = noteDict["creditorAddress"] as? [String: Any],
                          let amount = noteDict["amount"] as? Int,
                          let assetId = noteDict["assetId"] as? Int else {
                        throw NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid note format"])
                    }
                    
                    func convertAddress(_ addressDict: [String: Any]) throws -> AddressData {
                        guard let diversifier = (addressDict["diversifier"] as? [Int])?.map({ UInt8($0) }),
                              let transmissionKey = (addressDict["transmissionKey"] as? [Int])?.map({ UInt8($0) }),
                              let clueKey = (addressDict["clueKey"] as? [Int])?.map({ UInt8($0) }) else {
                            throw NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid address format"])
                        }
                        
                        return AddressData(
                            diversifier: Data(diversifier),
                            transmissionKey: Data(transmissionKey),
                            clueKey: Data(clueKey)
                        )
                    }
                    
                    return Note(
                        debtorAddress: try convertAddress(debtorAddressDict),
                        creditorAddress: try convertAddress(creditorAddressDict),
                        amount: UInt64(amount),
                        assetId: UInt64(assetId),
                        commitment: Data(commitment)
                    )
                }
                
                let note = try convertNote(noteDict)
                let result = try pm.signNote(seedPhrase: seedPhrase, note: note)
                
                promise.resolve([
                    "signature": Array(result.signature),
                    "verificationKey": Array(result.verificationKey),
                    "note": [
                        "commitment": Array(result.note.commitment),
                        "debtorAddress": [
                            "diversifier": Array(result.note.debtorAddress.diversifier),
                            "transmissionKey": Array(result.note.debtorAddress.transmissionKey),
                            "clueKey": Array(result.note.debtorAddress.clueKey)
                        ],
                        "creditorAddress": [
                            "diversifier": Array(result.note.creditorAddress.diversifier),
                            "transmissionKey": Array(result.note.creditorAddress.transmissionKey),
                            "clueKey": Array(result.note.creditorAddress.clueKey)
                        ]
                    ]
                ])
            } catch {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: error.localizedDescription]))
            }
        }
        
        AsyncFunction("verifySignature") { (args: [String: Any], promise: Promise) in
            guard let pm = self.proofManager else {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "ProofManager not initialized"]))
                return
            }
            
            do {
                guard let verificationKey = (args["verificationKey"] as? [Int])?.map({ UInt8($0) }),
                      let commitment = (args["commitment"] as? [Int])?.map({ UInt8($0) }),
                      let signature = (args["signature"] as? [Int])?.map({ UInt8($0) }) else {
                    throw NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid arguments"])
                }
                
                let result = try pm.verifySignature(
                    verificationKeyBytes: Data(verificationKey),
                    commitment: Data(commitment),
                    signature: Data(signature)
                )
                
                promise.resolve(result)
            } catch {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: error.localizedDescription]))
            }
        }
        
        AsyncFunction("createIntentAction") { (args: [String: Any], promise: Promise) in
            guard let pm = self.proofManager else {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "ProofManager not initialized"]))
                return
            }
            
            do {
                guard let debtorSeedPhrase = (args["debtorSeedPhrase"] as? [Int])?.map({ UInt8($0) }),
                      let rseedRandomness = (args["rseedRandomness"] as? [Int])?.map({ UInt8($0) }),
                      let debtorIndex = args["debtorIndex"] as? Int,
                      let creditorAddr = args["creditorAddr"] as? String,
                      let amount = args["amount"] as? Int,
                      let assetId = args["assetId"] as? Int else {
                    throw NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid arguments"])
                }
                
                let result = try pm.createIntentAction(
                    debtorSeedPhrase: Data(debtorSeedPhrase),
                    rseedRandomness: Data(rseedRandomness),
                    debtorIndex: UInt32(debtorIndex),
                    creditorAddr: creditorAddr,
                    amount: UInt64(amount),
                    assetId: UInt64(assetId)
                )
                
                promise.resolve(result)
            } catch {
                promise.reject(NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: error.localizedDescription]))
            }
        }
    }
}