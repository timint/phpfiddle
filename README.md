![preview](https://user-images.githubusercontent.com/359192/216814118-6a75c025-5687-4251-b70a-73704cebb096.png)

# PHP Fiddle for Windows

PHP Fiddle is a Windows desktop application that allows you to execute PHP code on the fly. Excellent for testing, learning, and experimenting without having to install a web server or IDE environment. Suitable for both beginners and professionals of all ages looking for a fast way to execute code. Created by swedish PHP developer T. Almroth.

**[Download Now](https://github.com/timint/phpfiddle/releases)**

## Questions & Answers

**Where is PHP Fiddle installed?**
-- PHP Fiddle is by default installed in %LocalAppData%\Programs\php-fiddle.

**Can I enable more PHP extensions or edit the PHP.ini configuration?**
-- Yes, make any changes to %LocalAppData%\Programs\php-fiddle\resources\app\php\php.ini and restart the application.

**Can I update the PHP version?**
-- Yes, you can grab a new release of PHP at the [PHP for Windows](https://windows.php.net/download/) website and replace the folder at %LocalAppData%\Programs\php-fiddle\resources\app\php\.

## Credits

The project is built using [Tauri](https://tauri.app/) (Rust) with a web frontend — Node.js is used for frontend tooling.

The PHP logotype is property of [The PHP Group](https://www.php.net/credits.php).

The javascript source code editor bundled with this software is [CodeMirror](https://codemirror.net/).

## How To Build

## FastCGI mode

This build uses PHP in FastCGI mode (php-cgi / php-fpm) and an internal HTTP → FastCGI proxy. By default the app spawns `php-cgi -b 127.0.0.1:9000` and the proxy listens on the port configured in `ui/config.json` (the `php.fastcgi_port` and `php.port` fields). Ensure `php-cgi` (or an equivalent FastCGI-capable PHP binary) is present in the supplied `ui/php` directory or available on PATH.



Make sure you have Node.js v22+ installed and the Rust toolchain (required by Tauri). Then follow these steps to build PHP Fiddle from source:

```bash
# Clone the project and navigate to the directory
git clone https://github.com/timint/phpfiddle.git && cd phpfiddle

# Install Node dependencies
npm install

# Install Rust toolchain (if missing)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable

# Install the Tauri CLI (globally optional, npx works too)
npm install -D @tauri-apps/cli
# or: cargo install tauri-cli

# Run the app in dev mode (starts PHP server and opens the Tauri window)
npm run start

# Build native bundles (requires platform-specific tools)
npm run build
```

# How To Run On Linux

```
# Update APT resources
sudo apt update

# Install dependencies
sudo apt install libfuse2 php

# Remove app restriction
sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0

# Run the app
./PHPFiddle.AppImage
```
