package com.wingdrive.core

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import expo.modules.kotlin.Promise

class WingMobileCoreModule : Module() {
    private var listeners = 0
    private var logListeners = 0
    private var registeredWithRust = false
    private var logRegisteredWithRust = false

    init {
        try {
            System.loadLibrary("wing_mobile_core")
        } catch (e: UnsatisfiedLinkError) {
            android.util.Log.e("WingMobileCore", "Failed to load native library: ${e.message}")
        }
    }

    override fun definition() = ModuleDefinition {
        Name("WingMobileCore")

        Events("WingCoreEvent", "WingCoreLog")

        OnStartObserving("WingCoreEvent") {
            android.util.Log.i("WingMobileCore", "📡 OnStartObserving WingCoreEvent triggered")

            if (!registeredWithRust) {
                try {
                    android.util.Log.i("WingMobileCore", "🚀 Registering event listener...")
                    registerCoreEventListener()
                    registeredWithRust = true
                    android.util.Log.i("WingMobileCore", "✅ Event listener registered with Rust")
                } catch (e: Exception) {
                    android.util.Log.e("WingMobileCore", "Failed to register event listener: ${e.message}")
                }
            }

            listeners++
            android.util.Log.i("WingMobileCore", "📊 WingCoreEvent listeners: $listeners")
        }

        OnStopObserving("WingCoreEvent") {
            listeners--
            android.util.Log.i("WingMobileCore", "📉 WingCoreEvent listeners: $listeners")
        }

        OnStartObserving("WingCoreLog") {
            android.util.Log.i("WingMobileCore", "📡 OnStartObserving WingCoreLog triggered")

            if (!logRegisteredWithRust) {
                try {
                    android.util.Log.i("WingMobileCore", "🚀 Registering log listener...")
                    registerCoreLogListener()
                    logRegisteredWithRust = true
                    android.util.Log.i("WingMobileCore", "✅ Log listener registered with Rust")
                } catch (e: Exception) {
                    android.util.Log.e("WingMobileCore", "Failed to register log listener: ${e.message}")
                }
            }

            logListeners++
            android.util.Log.i("WingMobileCore", "📊 WingCoreLog listeners: $logListeners")
        }

        OnStopObserving("WingCoreLog") {
            logListeners--
            android.util.Log.i("WingMobileCore", "📉 WingCoreLog listeners: $logListeners")
        }

        Function("initialize") { dataDir: String?, deviceName: String? ->
            val dir = dataDir ?: appContext.persistentFilesDirectory?.absolutePath
                ?: throw Exception("No data directory available")

            try {
                initializeCore(dir, deviceName)
            } catch (e: Exception) {
                android.util.Log.e("WingMobileCore", "Failed to initialize core: ${e.message}")
                -1
            }
        }

        AsyncFunction("sendMessage") { query: String, promise: Promise ->
            try {
                handleCoreMsg(query, WingCorePromise(promise))
            } catch (e: Exception) {
                promise.reject("CORE_ERROR", e.message ?: "Unknown error", e)
            }
        }

        Function("shutdown") {
            try {
                shutdownCore()
            } catch (e: Exception) {
                android.util.Log.e("WingMobileCore", "Failed to shutdown core: ${e.message}")
            }
        }
    }

    fun getDataDirectory(): String {
        return appContext.persistentFilesDirectory?.absolutePath ?: ""
    }

    fun sendCoreEvent(body: String) {
        if (listeners > 0) {
            this@WingMobileCoreModule.sendEvent("WingCoreEvent", mapOf("body" to body))
        }
    }

    fun sendCoreLog(body: String) {
        if (logListeners > 0) {
            this@WingMobileCoreModule.sendEvent("WingCoreLog", mapOf("body" to body))
        }
    }

    // Native methods - will throw UnsatisfiedLinkError if library not loaded
    private external fun registerCoreEventListener()
    private external fun registerCoreLogListener()
    private external fun initializeCore(dataDir: String, deviceName: String?): Int
    private external fun handleCoreMsg(query: String, promise: WingCorePromise)
    private external fun shutdownCore()
}

class WingCorePromise(private val promise: Promise) {
    fun resolve(msg: String) {
        promise.resolve(msg)
    }

    fun reject(error: String) {
        promise.reject("CORE_ERROR", error, null)
    }
}
