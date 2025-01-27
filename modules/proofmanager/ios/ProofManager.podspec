Pod::Spec.new do |s|
  s.name           = 'ProofManager'
  s.version        = '1.0.0'
  s.summary        = 'Zero Knowledge Proof Manager'
  s.description    = 'ZK proof generation and validation'
  s.author         = ''
  s.homepage       = 'https://docs.expo.dev/modules/'
  s.platform       = :ios, '13.0'
  s.source         = { git: '' }
  s.static_framework = true
  
  # Change to use the XCFramework instead of the static library
  s.ios.vendored_frameworks = 'ProofManager.xcframework'
  
  s.source_files = "*.{h,m,mm,swift,hpp,cpp}"
  s.dependency 'ExpoModulesCore'
  
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'SWIFT_COMPILATION_MODE' => 'wholemodule'
  }
end