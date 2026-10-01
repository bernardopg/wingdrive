import {
  NativeModule,
  requireNativeModule,
  EventEmitter,
} from "expo-modules-core";

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

export interface CoreModule {
  initialize(dataDir?: string, deviceName?: string): Promise<number>;
  sendMessage(query: string): Promise<string>;
  shutdown(): void;
  addListener(callback: (event: CoreEvent) => void): () => void;
  addLogListener(callback: (log: CoreLog) => void): () => void;
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
};
