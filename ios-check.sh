#!/bin/bash

# Color coding for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🔍 Starting iOS build environment diagnostics..."

# Check Xcode installation and version
echo -e "\n📱 Checking Xcode..."
xcodebuild -version
xcrun --sdk iphoneos --show-sdk-path

# Check React Native environment
echo -e "\n⚛️ Checking React Native environment..."
echo "Node version: $(node -v)"
echo "npm version: $(npm -v)"
echo "Expo CLI version: $(expo --version)"
echo "Ruby version: $(ruby -v)"
echo "CocoaPods version: $(pod --version)"

# Check architecture-specific settings
echo -e "\n🏗 Checking architecture settings..."
echo "Machine architecture: $(uname -m)"
echo "Supported architectures in project:"
xcrun lipo -info ./ios/build/Build/Products/Debug-iphonesimulator/*.app 2>/dev/null || echo "No build products found"

# Check Podfile.lock for architecture-specific configurations
echo -e "\n📦 Checking Podfile.lock..."
if [ -f "ios/Podfile.lock" ]; then
    echo "Podfile.lock exists"
    grep -i "platform" ios/Podfile.lock
    grep -i "architecture" ios/Podfile.lock
else
    echo -e "${RED}Podfile.lock not found${NC}"
fi

# Check build settings
echo -e "\n⚙️ Checking Xcode build settings..."
xcodebuild -project ios/*.xcodeproj -showBuildSettings 2>/dev/null | grep -i "ARCH"

# Check for common problem indicators
echo -e "\n🚨 Checking for common issues..."

# Check pod installation
if [ -d "ios/Pods" ]; then
    echo -e "${GREEN}✓ Pods directory exists${NC}"
else
    echo -e "${RED}✗ Pods directory missing${NC}"
fi

# Check for architecture-specific build settings in project.pbxproj
echo -e "\n📝 Checking project.pbxproj for architecture settings..."
grep -i "ARCHS" ios/*.xcodeproj/project.pbxproj

echo -e "\n✅ Diagnostic complete!"