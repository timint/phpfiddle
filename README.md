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

The project is built with Node.js using [Electron](https://www.electronjs.org/).

The PHP logotype is property of [The PHP Group](https://www.php.net/credits.php).

The javascript source code editor bundled with this software is [CodeMirror](https://codemirror.net/).

## How To Build

Make sure you have Node.js v22+ installed and follow these steps to build PHP Fiddle from source:

```bash
# Clone the project and navigate to the directory
git clone https://github.com/timint/phpfiddle.git && cd phpfiddle

# Install dependencies
npm install

# Simulate running the app
npm run start

# Build and compile app to dist/ folder
npm run build
```
