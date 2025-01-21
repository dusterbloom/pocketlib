const { withProjectBuildGradle, createRunOncePlugin } = require('@expo/config-plugins');

const withProofManager = (config) => {
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

module.exports = createRunOncePlugin(withProofManager, 'proofmanager', '1.0.0');