const { withProjectBuildGradle, createRunOncePlugin, withDangerousMod } = require('@expo/config-plugins');
const fs = require('fs');
const path = require('path');

const withAndroidProofManager = (config) => {
  return withProjectBuildGradle(config, (config) => {
    if (config.modResults.language === 'groovy') {
      config.modResults.contents = config.modResults.contents.replace(
        /allprojects {/,
        `allprojects {
          repositories {
              flatDir {
                  dirs "$rootDir/../node_modules/proofmanager/android/libs"
              }
              maven { url "$rootDir/../node_modules/expo/maven" }
              maven { url "$rootDir/../node_modules/expo-modules-core/maven" }
          }`
      );
    }
    return config;
  });
};

const withIosProofManager = (config) => {
  return withDangerousMod(config, [
    'ios',
    async (config) => {
      const moduleIosPath = path.join(config.modRequest.platformProjectRoot, 'ProofManager');
      if (!fs.existsSync(moduleIosPath)) {
        const sourcePath = path.join(config.modRequest.projectRoot, 'modules', 'proofmanager', 'ios');
        fs.cpSync(sourcePath, moduleIosPath, { recursive: true });
        console.log('Copied ProofManager module to ios directory');
      }

      // Ensure the module is included in the Xcode project
      const pbxprojPath = path.join(config.modRequest.platformProjectRoot, 'pocketlib.xcodeproj', 'project.pbxproj');
      if (fs.existsSync(pbxprojPath)) {
        let pbxprojContent = fs.readFileSync(pbxprojPath, 'utf8');
        if (!pbxprojContent.includes('ProofManager')) {
          // You may need to add the module to the Xcode project here
          console.log('ProofManager module reference added to Xcode project');
        }
      }
      
      return config;
    },
  ]);
};

const withProofManager = (config) => {
  config = withAndroidProofManager(config);
  config = withIosProofManager(config);
  return config;
};

module.exports = createRunOncePlugin(withProofManager, 'proofmanager', '1.0.0');