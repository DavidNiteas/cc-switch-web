export function getCurrentWindow() {
  return {
    setTitle: async () => {},
    minimize: async () => {},
    maximize: async () => {},
    unmaximize: async () => {},
    toggleMaximize: async () => {},
    show: async () => {},
    hide: async () => {},
    close: async () => {},
    setSize: async () => {},
    setPosition: async () => {},
    setResizable: async () => {},
    setAlwaysOnTop: async () => {},
    setFullscreen: async () => {},
    isFullscreen: async () => false,
    isMaximized: async () => false,
    onResized: async () => () => {},
    onMoved: async () => () => {},
    onCloseRequested: async () => () => {},
    onThemeChanged: async () => () => {},
  };
}

export function getCurrentWebviewWindow() {
  return getCurrentWindow();
}
