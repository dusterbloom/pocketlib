// modules/proofmanager/plugin.js
const { createRunOncePlugin } = require('@expo/config-plugins');

const withProofManager = (config) => {
  return config;
};

module.exports = createRunOncePlugin(
  withProofManager,
  'proofmanager',
  '1.0.0'
);