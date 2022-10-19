const path = require('path');

const { app, BrowserWindow,dialog,ipcMain,protocol } = require('electron');
const isDev = require('electron-is-dev');

let mainWindow;

function createWindow() {
  const win = new BrowserWindow({
    autoHideMenuBar:true,

    width: 1100,
    height: 700,
    webPreferences: {
      nodeIntegration: true,
      
      preload: path.join(__dirname, 'preload.js'),
      webSecurity: false


    },
    icon: "public/logo512.png"

  });

   win.loadURL(
    isDev
      ? 'http://localhost:3000'
      : 'file://'+ __dirname +'/../build/index.html'
  );

  // Open the DevTools.
  if (isDev) {
    win.webContents.openDevTools({ mode: 'detach' });
  }

  return win;
}

async function handleFileOpen(event,options) {
  mainWindow.minimize();
  
  const { canceled, filePaths } = await dialog.showOpenDialog(BrowserWindow.getFocusedWindow(),options)
  
  mainWindow.restore();

  if (canceled) {

    return
  } else {

    return filePaths[0]
  }
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then( () =>{
  
  mainWindow = createWindow();

  ipcMain.handle('dialog:openFile', handleFileOpen)
  
});

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});


