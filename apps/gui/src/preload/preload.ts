import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("electron", {
  // HDQL Execution
  executeHDQL: (query: string) => ipcRenderer.invoke("execute-hdql", query),
  getCollections: () => ipcRenderer.invoke("get-collections"),
  getPortals: () => ipcRenderer.invoke("get-portals"),

  // File operations (for portal config editing)
  saveConfig: (portalId: string, config: any) =>
    ipcRenderer.invoke("save-config", portalId, config),
  loadConfig: (portalId: string) =>
    ipcRenderer.invoke("load-config", portalId),
});

declare global {
  interface Window {
    electron: {
      executeHDQL: (query: string) => Promise<any>;
      getCollections: () => Promise<any[]>;
      getPortals: () => Promise<any[]>;
      saveConfig: (portalId: string, config: any) => Promise<void>;
      loadConfig: (portalId: string) => Promise<any>;
    };
  }
}
