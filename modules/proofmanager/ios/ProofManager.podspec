# Pod::Spec.new do |s|
#     s.name           = 'ProofManager'
#     s.version        = '1.0.0'
#     s.summary        = 'Zero Knowledge Proof Manager'
#     s.description    = 'ZK proof generation and validation'
#     s.author         = ''
#     s.homepage       = 'https://docs.expo.dev/modules/'
#     s.platform       = :ios, '15.1'
#     s.source         = { git: '' }
#     s.static_framework = true
#     s.vendored_libraries = 'rust/libproofmanager.a'
#     s.dependency 'ExpoModulesCore'
#     s.pod_target_xcconfig = {
#       'DEFINES_MODULE' => 'YES',
#       'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386'
#     }
#     s.source_files = "**/*.{h,m,mm,swift,hpp,cpp}"
#   end

Pod::Spec.new do |s|
    s.name           = 'ProofManager'
    s.version        = '1.0.0'
    s.summary        = 'Zero Knowledge Proof Manager'
    s.description    = 'ZK proof generation and validation'
    s.author         = ''
    s.homepage       = 'https://docs.expo.dev/modules/'
    s.platform       = :ios, '15.1'
    s.source         = { git: '' }
    s.static_framework = true
    
    s.source_files = "**/*.{h,m,mm,swift,hpp,cpp}"
    s.dependency 'ExpoModulesCore'
    
    s.vendored_libraries = "ProofManager.xcframework/ios-arm64/libproofmanager.a"
    s.pod_target_xcconfig = {
      'DEFINES_MODULE' => 'YES',
      'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386'
    }
  end