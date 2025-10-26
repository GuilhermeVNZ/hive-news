import { app, BrowserWindow, ipcMain } from "electron";
import { join } from "path";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

let mainWindow: BrowserWindow | null = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    webPreferences: {
      preload: join(__dirname, "../preload/preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
    icon: join(__dirname, "../../assets/icon.png"),
  });

  // Load the UI
  if (process.env.NODE_ENV === "development") {
    mainWindow.loadURL("http://localhost:5173");
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(join(__dirname, "../renderer/index.html"));
  }
}

// IPC Handlers
ipcMain.handle("execute-hdql", async (_event, query: string) => {
  // TODO: Implement HDQL execution via MCP or HTTP API
  console.log("Executing HDQL query:", query);
  return {
    results: [],
    metadata: { executionTime: 0, resultCount: 0 },
  };
});

ipcMain.handle("get-collections", async () => {
  // TODO: Get available collections from Vectorizer
  return [
    { id: "news-content", name: "News Content", count: 0 },
    { id: "articles", name: "Articles", count: 0 },
  ];
});

ipcMain.handle("get-portals", async () => {
  // TODO: Get portal configurations
  return [
    { id: "airesearch", name: "AI Research", active: true },
    { id: "scienceai", name: "Science AI", active: true },
  ];
});

app.whenReady().then(createWindow);

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  if (mainWindow === null) {
    createWindow();
  }
});
