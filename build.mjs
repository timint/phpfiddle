import { argv } from 'process';
import os from 'os';
import { execSync } from 'child_process';

// Elevate to admin if not already (Windows only)
if (os.platform() === 'win32') {
  try {
    execSync('net session', { stdio: 'ignore' });
  } catch (error) {
    console.error('Error: This script requires administrative privileges. Please run as administrator.');
    process.exit(1);
  }
}

const validPlatforms = ['win', 'mac', 'linux'];

const platformMap = {
  win32: 'win',
  darwin: 'mac',
  linux: 'linux',
};

const currentPlatform = [platformMap[os.platform()]];

// Grab any unsupported arguments passed to the script
const invalidArgs = argv.slice(2).filter(arg => !validPlatforms.includes(arg));

if (invalidArgs.length > 0) {
  console.error(`Error: Invalid arguments: ${invalidArgs.join(', ')}. Use win, mac, or linux.\n`);
  process.exit(1);
}

const platforms = argv.slice(2).filter(arg => validPlatforms.includes(arg)).length > 0 ? argv.slice(2).filter(arg => validPlatforms.includes(arg)) : currentPlatform;

if (platforms.length === 0) {
  console.error('Error: No valid platforms specified. Use win, mac, or linux.');
  process.exit(1);
}

console.log(`Building for platforms: ${platforms.join(', ')}`);

for (const platform of platforms) {
  if (platform === 'mac' && os.platform() !== 'darwin') {
    console.error('Error: Building for Mac requires a macOS environment.');
    process.exit(1);
  }
}

// Run Tauri build (relies on Rust toolchain + @tauri-apps/cli)
try {
  console.log('Running `tauri build`...');
  execSync('npx tauri build', { stdio: 'inherit' });
  console.log('Tauri build completed successfully.');
} catch (error) {
  console.error('Tauri build failed:', error);
  process.exit(1);
}
