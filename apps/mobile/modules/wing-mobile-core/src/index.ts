import {
  NativeModule,
  requireNativeModule,
  EventEmitter,
} from "expo";

export interface CoreEvent {
  body: string;
}
export interface LogMessage {
  timestamp: string;
  level: string;
  target: string;
  message: string;
  job_id?: string;
  library_id?: string;
}
export interface CoreLog {
  body: string;
}

type WingMobileCoreEvents = {
  WingCoreEvent: (event: CoreEvent) => void;
  WingCoreLog: (log: CoreLog) => void;
};

/** State of background sync on Android (MOB-018) */
export interface BackgroundSyncState {
  /** False on platforms without a background service (iOS) */
  supported: boolean;
  /** The user's choice, kept across restarts */
  enabled: boolean;
  /** Whether the foreground service runs now */
  running: boolean;
  /** Whether Android exempts the app from battery optimization */
  unrestrictedBattery: boolean;
  /** Whether the service's notification can be shown */
  notificationsAllowed: boolean;
}

export const UNSUPPORTED_BACKGROUND_SYNC: BackgroundSyncState = {
  supported: false,
  enabled: false,
  running: false,
  unrestrictedBattery: false,
  notificationsAllowed: false,
};

export interface CoreModule {
  initialize(dataDir?: string, deviceName?: string): Promise<number>;
  sendMessage(query: string): Promise<string>;
  shutdown(): void;
  addListener(callback: (event: CoreEvent) => void): () => void;
  addLogListener(callback: (log: CoreLog) => void): () => void;
  /** Whether the app may read shared storage (Android); always true elsewhere */
  hasAllFilesAccess(): boolean;
  /** Opens the system page that grants storage access; false when it could not */
  openAllFilesAccessSettings(): boolean;
  /** Opens a file of this device in another app; false when none handles it or on iOS */
  openFile(path: string, mimeType?: string): boolean;
  /** Shows the share sheet for a file of this device; false on iOS */
  shareFile(path: string, mimeType?: string): boolean;
  /** Whether this platform can open and share files (Android only for now) */
  canOpenFiles: boolean;
  /** Turns background sync on or off; true when the service runs afterwards */
  setBackgroundSync(enabled: boolean): boolean;
  getBackgroundSync(): BackgroundSyncState;
  /** Asks Android to stop restricting the app's battery use; false when it could not */
  openBatteryOptimizationSettings(): boolean;
}

interface WingMobileCoreNativeModule extends NativeModule<WingMobileCoreEvents> {
  initialize(
    dataDir: string | null,
    deviceName: string | null,
  ): Promise<number>;
  sendMessage(query: string): Promise<string>;
  shutdown(): void;
  addListener(callback: (event: CoreEvent) => void): () => void;
  addLogListener(callback: (log: CoreLog) => void): () => void;
  hasAllFilesAccess?(): boolean;
  openAllFilesAccessSettings?(): boolean;
  openFile?(path: string, mimeType: string | null): boolean;
  shareFile?(path: string, mimeType: string | null): boolean;
  setBackgroundSync?(enabled: boolean): boolean;
  getBackgroundSync?(): BackgroundSyncState;
  openBatteryOptimizationSettings?(): boolean;
}

const WingMobileCoreModule =
  requireNativeModule<WingMobileCoreNativeModule>("WingMobileCore");

if (!WingMobileCoreModule) {
  throw new Error("WingMobileCoreModule has not been initialized. Did you run 'cargo xtask build-mobile' and rebuild the app?")
}

const emitter = new EventEmitter<WingMobileCoreEvents>(WingMobileCoreModule as any);

export const WingMobileCore: CoreModule = {
  initialize: async (dataDir?: string, deviceName?: string) => {
    return WingMobileCoreModule.initialize(dataDir ?? null, deviceName ?? null);
  },
  sendMessage: async (query: string) => {
    return WingMobileCoreModule.sendMessage(query);
  },
  shutdown: () => {
    WingMobileCoreModule.shutdown();
  },
  addListener: (callback: (event: CoreEvent) => void) => {
    const subscription = emitter.addListener("WingCoreEvent", callback);
    return () => subscription.remove();
  },
  addLogListener: (callback: (log: CoreLog) => void) => {
    const subscription = emitter.addListener("WingCoreLog", callback);
    return () => subscription.remove();
  },
  // iOS has no such permission, so only Android implements these.
  hasAllFilesAccess: () => WingMobileCoreModule.hasAllFilesAccess?.() ?? true,
  openAllFilesAccessSettings: () =>
    WingMobileCoreModule.openAllFilesAccessSettings?.() ?? false,
  openFile: (path: string, mimeType?: string) =>
    WingMobileCoreModule.openFile?.(path, mimeType ?? null) ?? false,
  shareFile: (path: string, mimeType?: string) =>
    WingMobileCoreModule.shareFile?.(path, mimeType ?? null) ?? false,
  canOpenFiles: typeof WingMobileCoreModule.openFile === "function",
  setBackgroundSync: (enabled: boolean) =>
    WingMobileCoreModule.setBackgroundSync?.(enabled) ?? false,
  getBackgroundSync: () =>
    WingMobileCoreModule.getBackgroundSync?.() ?? UNSUPPORTED_BACKGROUND_SYNC,
  openBatteryOptimizationSettings: () =>
    WingMobileCoreModule.openBatteryOptimizationSettings?.() ?? false,
};
