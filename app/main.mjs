import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { readFileSync } from 'fs';
import PHPServer from './server.mjs';
import electron from 'electron';

const _dirname = dirname(fileURLToPath(import.meta.url));
const configFile = join(_dirname, 'config.json');
const config = JSON.parse(readFileSync(configFile, 'utf-8'));

const { app, Menu, BrowserWindow } = electron;

const phpRuntime = new PHPServer({
  php: join(_dirname, config.php.directory, 'php'),
  directory: join(_dirname, config.php.docroot),
  port: config.php.port,
  stdio: 'ignore',
  directives: config.php.directives,
});

let mainWindow;
function createWindow() {
  let runtimePromise = phpRuntime.run();

  mainWindow = new BrowserWindow({
  	width: config.window.width,
  	height: config.window.height
  });
  mainWindow.loadURL('http://'+phpRuntime.host+':'+phpRuntime.port+'/');

  //mainWindow.webContents.openDevTools()

  mainWindow.on('closed', function () {
    phpRuntime.close();
    mainWindow = null;
  })
}

app.on('ready', () => {

  // Set the application menu to null to remove it
  Menu.setApplicationMenu(null);

  // Create the main window
  createWindow();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    phpRuntime.close();
    app.quit();
  }
});

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  }
});