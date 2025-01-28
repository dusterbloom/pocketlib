Pod::Spec.new do |s|
  s.name           = 'ProofManager'
  s.version        = '1.0.0'
  s.summary        = 'Zero Knowledge Proof Manager'
  s.description    = 'ZK proof generation and validation'
  s.author         = ''
  s.homepage       = 'https://docs.expo.dev/modules/'
  s.platform       = :ios, '18.2'
  s.source         = { git: '' }
  s.static_framework = true
  
  s.dependency 'ExpoModulesCore'

  # Change to use the XCFramework instead of the static library
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'SWIFT_COMPILATION_MODE' => 'wholemodule',
    'EXCLUDED_ARCHS' => 'x86_64',
    'LIBRARY_SEARCH_PATHS' => [
            '$(PODS_TARGET_SRCROOT)/Proof.xcframework/ios-arm64',
            '$(PODS_TARGET_SRCROOT)/Proof.xcframework/ios-arm64-simulator'
        ],
        'HEADER_SEARCH_PATHS' => [
            '$(PODS_TARGET_SRCROOT)/Proof.xcframework/ios-arm64/Headers',
            '$(PODS_TARGET_SRCROOT)/Proof.xcframework/ios-arm64-simulator/Headers'
        ],
        'MODULEMAP_FILE' => '$(PODS_TARGET_SRCROOT)/module.modulemap'
  }

  
  s.source_files = "*.{h,m,mm,swift,hpp,cpp}"
  s.ios.vendored_frameworks = 'ProofManager.xcframework'
end


# Peunmbra style - > ld: symbol(s) not found for architecture arm64
# Pod::Spec.new do |s|
#   s.name           = 'ProofManager'
#   s.version        = '1.0.0'
#   s.summary        = 'Proof generation and verification module'
#   s.description    = 'Expo module for proof generation and verification'
#   s.author         = ''
#   s.homepage       = 'https://docs.expo.dev/modules/'
#   s.platform       = :ios, '13.0'
#   s.source         = { git: '' }
#   s.static_framework = true

#   s.dependency 'ExpoModulesCore'
  
#   s.vendored_frameworks = 'ProofManager.xcframework'
#   s.source_files = ["*.{h,m,mm,swift}"]
  
#   s.pod_target_xcconfig = {
#     'DEFINES_MODULE' => 'YES',
#     'SWIFT_COMPILATION_MODE' => 'wholemodule',
#     'ENABLE_BITCODE' => 'NO',
#     'CLANG_CXX_LANGUAGE_STANDARD' => 'c++20',
#     'CLANG_CXX_LIBRARY' => 'libc++',
#     'MODULEMAP_FILE' => '$(PODS_ROOT)/../../modules/proofmanager/ios/ProofManager.xcframework/ios-arm64-simulator/Headers/module.modulemap',
#     'LIBRARY_SEARCH_PATHS' => [
#       '$(PODS_ROOT)/../../modules/proofmanager/ios/ProofManager.xcframework/ios-arm64-simulator',
#       '$(PODS_ROOT)/../../modules/proofmanager/ios/ProofManager.xcframework/ios-arm64'
#     ],
#     'FRAMEWORK_SEARCH_PATHS' => [
#       '$(PODS_ROOT)/../../modules/proofmanager/ios'
#     ],
#     'OTHER_LDFLAGS' => '-ObjC -all_load -force_load "$(PODS_ROOT)/../../modules/proofmanager/ios/ProofManager.xcframework/ios-arm64-simulator/libproofmanager.a"',
#     'SWIFT_INCLUDE_PATHS' => [
#       '$(PODS_ROOT)/../../modules/proofmanager/ios/ProofManager.xcframework/ios-arm64-simulator/Headers'
#     ]
#   }

#   s.preserve_paths = 'ProofManager.xcframework/**/*'
# end

# Pod::Spec.new do |s|
#   s.name           = 'ProofManager'
#   s.version        = '1.0.0'
#   s.summary        = 'Proof generation and verification module'
#   s.description    = 'Expo module for proof generation and verification'
#   s.author         = ''
#   s.homepage       = 'https://docs.expo.dev/modules/'
#   s.platform       = :ios, '13.0'
#   s.source         = { git: '' }
#   s.static_framework = true

#   s.dependency 'ExpoModulesCore'
  
#   s.pod_target_xcconfig = {
#     'DEFINES_MODULE' => 'YES',
#     'SWIFT_COMPILATION_MODE' => 'wholemodule',
#     'VALID_ARCHS' => 'arm64',  # Only supporting arm64
#     'EXCLUDED_ARCHS' => 'x86_64 i386',  # Explicitly exclude x86
#   }

#   s.vendored_frameworks = 'ProofManager.xcframework'
#   s.source_files = "*.{h,m,mm,swift,hpp,cpp}" 
# end