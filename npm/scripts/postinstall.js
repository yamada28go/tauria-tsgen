// scripts/postinstall.js
const fs = require('fs');
const path = require('path');
const { platform } = process;

const packageName = 'tauria-tsgen'; // package.jsonのnameと合わせる

const binaryPath = path.join(__dirname, '..', 'bin', packageName + (platform === 'win32' ? '.exe' : ''));

if (platform !== 'win32') {
  // Windows以外では実行権限を付与
  try {
    fs.chmodSync(binaryPath, '755'); // rwxr-xr-x
    console.log(`Set executable permissions for ${binaryPath}`);
  } catch (error) {
    console.error(`Failed to set executable permissions for ${binaryPath}: ${error.message}`);
    process.exit(1);
  }
} else {
  console.log('No executable permissions needed for Windows.');
}