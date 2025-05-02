import { build, Platform, Arch } from 'electron-builder';
import { argv } from 'process';
import { readFileSync } from 'fs';
import os from 'os';
import { execSync } from 'child_process';

// Elevate to admin if not already
if (os.platform() === 'win32') {
  try {
    execSync('net session', { stdio: 'ignore' });
  } catch (error) {
    console.error('Error: This script requires administrative privileges. Please run as administrator.');
    process.exit(1);
  }
}

const packageJson = JSON.parse(readFileSync('./package.json', 'utf-8'));

const validPlatforms = ['win', 'mac', 'linux'];

const platformMap = {
  win32: 'win',
  darwin: 'mac',
  linux: 'linux',
};

const currentPlatform = [platformMap[os.platform()]];

// Grab any unsupported arguments passed to the script
const invalidArgs = argv.slice(2).filter(arg => !validPlatforms.includes(arg));

// Check if the all of the arguments are valid platform keys, halt if not
if (invalidArgs.length > 0) {
  console.error(`Error: Invalid arguments: ${invalidArgs.join(', ')}. Use win, mac, or linux.\n`);
  process.exit(1);
}

const platforms = argv.slice(2).filter(arg => validPlatforms.includes(arg)).length > 0 ? argv.slice(2).filter(arg => validPlatforms.includes(arg)) : currentPlatform;

if (platforms.length === 0) {
  console.error('Error: No valid platforms specified. Use win, mac, or linux.');
  process.exit(1);
}

// Clean up the dist directory before building
//execSync('rm -rf dist', { stdio: 'ignore' });
//execSync('mkdir dist', { stdio: 'ignore' });

console.log(`Building for platforms: ${platforms.join(', ')}`);

(async () => {
  for (const platform of platforms) {
    let targets;
    switch (platform) {
      case 'win':
        console.log('Building for Windows...');
        targets = Platform.WINDOWS.createTarget('nsis', Arch.x64);
        break;
      case 'mac':
        if (os.platform() !== 'darwin') {
          console.error('Error: Building for Mac requires a macOS environment.');
          process.exit(1);
        }
        console.log('Building for Mac...');
        targets = Platform.MAC.createTarget('dmg', Arch.x64);
        break;
      case 'linux':
        console.log('Building for Linux...');
        targets = Platform.LINUX.createTarget('AppImage', Arch.x64);
        break;
      default:
        console.error(`Error: Unsupported platform: ${platform}`);
        process.exit(1);
    }
    try {
      await build({ targets});
      console.log(`Build process for ${platform} completed successfully.`);
    } catch (error) {
      console.error(`Error: ${error.message}`);
      process.exit(1);
    }
  }
})();
