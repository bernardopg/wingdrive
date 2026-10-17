//! WingDrive Mobile Core FFI Layer
//!
//! This module provides the FFI bridge between React Native (via Expo modules)
//! and the WingDrive core. It's adapted from the V2 iOS core implementation
//! to work with both iOS and Android.

use std::{
	ffi::{CStr, CString},
	path::PathBuf,
	sync::Arc,
};

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use tracing::{error, info, warn};
use uuid::Uuid;

use wing_core::{
	infra::daemon::rpc::RpcServer,
	infra::daemon::types::{DaemonError, DaemonRequest, DaemonResponse},
	infra::event::log_emitter::{set_global_log_bus, LogEventLayer},
	Core,
};

// Global state for embedded core
static RUNTIME: OnceCell<Runtime> = OnceCell::new();
static CORE: OnceCell<Arc<Core>> = OnceCell::new();

// JSON-RPC protocol types
#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
	jsonrpc: String,
	method: String,
	params: JsonRpcParams,
	id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcParams {
	input: serde_json::Value,
	library_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
	jsonrpc: String,
	id: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	result: Option<serde_json::Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
	code: i32,
	message: String,
}

/// Initialize the embedded core with full WingDrive functionality
///
/// # Safety
/// `data_dir` must be a valid null-terminated C string. `device_name` may be null.
#[no_mangle]
pub unsafe extern "C" fn initialize_core(
	data_dir: *const std::os::raw::c_char,
	device_name: *const std::os::raw::c_char,
) -> std::os::raw::c_int {
	let data_dir_str = unsafe { CStr::from_ptr(data_dir).to_string_lossy().to_string() };

	let device_name_opt = if device_name.is_null() {
		None
	} else {
		let name = unsafe { CStr::from_ptr(device_name).to_string_lossy().to_string() };
		if name.is_empty() {
			None
		} else {
			Some(name)
		}
	};

	init_tracing();

	info!(
		"Initializing embedded WingDrive core with data dir: {}, device name: {:?}",
		data_dir_str, device_name_opt
	);

	// Check if already initialized (singleton pattern)
	if RUNTIME.get().is_some() && CORE.get().is_some() {
		info!("Embedded core already initialized, skipping");
		return 0;
	}

	// Initialize Tokio runtime
	let rt = match Runtime::new() {
		Ok(rt) => rt,
		Err(e) => {
			error!("Failed to create Tokio runtime: {}", e);
			return -1;
		}
	};

	// Ensure data directory exists
	let data_path = PathBuf::from(data_dir_str.clone());
	if let Err(e) = std::fs::create_dir_all(&data_path) {
		error!("Failed to create data directory: {}", e);
		return -1;
	}

	// Initialize core
	let core = rt.block_on(async { Core::new_with_config(data_path, None, device_name_opt).await });

	let mut core = match core {
		Ok(core) => core,
		Err(e) => {
			error!("Failed to initialize core: {}", e);
			return -1;
		}
	};

	// Initialize networking with protocol registration
	match rt.block_on(core.init_networking()) {
		Ok(()) => info!("Networking initialized with protocol registration"),
		Err(e) => warn!(
			"Failed to initialize networking, pairing will not work: {}",
			e
		),
	}

	let core = Arc::new(core);

	// Set global log bus for log streaming
	set_global_log_bus(core.logs.clone());

	// Store global state
	let _ = RUNTIME.set(rt);
	let _ = CORE.set(core);

	info!("Mobile core initialized successfully");

	0 // Success
}

/// Installs the tracing subscriber once. On Android stdout is discarded, so the fmt layer writes to
/// logcat under the `WingDriveCore` tag instead.
fn init_tracing() {
	use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

	let fmt_layer = tracing_subscriber::fmt::layer().with_ansi(false);
	#[cfg(target_os = "android")]
	let fmt_layer = fmt_layer
		.without_time()
		.with_writer(android::LogcatWriter::default);

	let _ = tracing_subscriber::registry()
		.with(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,wing_core=debug")),
		)
		.with(fmt_layer)
		.with(LogEventLayer::new())
		.try_init();
}

/// Shutdown the embedded core
#[no_mangle]
pub extern "C" fn shutdown_core() {
	// The core lives until the process exits; there is nothing to release yet.
	info!("Shutdown requested for the embedded core");
}

/// Handle JSON-RPC message from the embedded core
///
/// # Safety
/// - `query` must be a valid, non-null, null-terminated C string.
/// - `callback` must be a valid function pointer that is safe to call from any thread.
/// - `callback_data` must remain valid until `callback` is invoked (invocation is asynchronous).
#[no_mangle]
pub unsafe extern "C" fn handle_core_msg(
	query: *const std::os::raw::c_char,
	callback: extern "C" fn(*mut std::os::raw::c_void, *const std::os::raw::c_char),
	callback_data: *mut std::os::raw::c_void,
) {
	let query_str = unsafe { CStr::from_ptr(query).to_string_lossy().to_string() };

	// RUNTIME is set before CORE, so both must be checked: a request can race initialization.
	let (Some(runtime), Some(core)) = (RUNTIME.get(), CORE.get()) else {
		callback(
			callback_data,
			cr#"{"jsonrpc":"2.0","id":"","error":{"code":-32603,"message":"Core not initialized"}}"#.as_ptr(),
		);
		return;
	};

	// Convert callback pointers to usize for Send safety
	let callback_fn_ptr: usize = callback as usize;
	let callback_data_int: usize = callback_data as usize;

	// Spawn async task to handle the request
	runtime.spawn(async move {
		let response = handle_json_rpc_request(query_str, core).await;
		let response_json = serde_json::to_string(&response).unwrap_or_else(|_|
			r#"{"jsonrpc":"2.0","id":"","error":{"code":-32603,"message":"Response serialization failed"}}"#.to_string()
		);

		let response_cstring = CString::new(response_json).unwrap();
		let callback: extern "C" fn(*mut std::os::raw::c_void, *const std::os::raw::c_char) =
			unsafe { std::mem::transmute(callback_fn_ptr) };
		let callback_data_ptr: *mut std::os::raw::c_void =
			callback_data_int as *mut std::os::raw::c_void;

		callback(callback_data_ptr, response_cstring.as_ptr());
	});
}

/// Start listening for core events using the real event system
#[no_mangle]
pub extern "C" fn spawn_core_event_listener(
	callback: extern "C" fn(*mut std::os::raw::c_void, *const std::os::raw::c_char),
	callback_data: *mut std::os::raw::c_void,
) {
	let (Some(runtime), Some(core)) = (RUNTIME.get(), CORE.get()) else {
		error!("Core not initialized, cannot start event listener");
		return;
	};

	let callback_fn_ptr: usize = callback as usize;
	let callback_data_int: usize = callback_data as usize;

	let mut event_subscriber = core.events.subscribe();

	runtime.spawn(async move {
		while let Ok(event) = event_subscriber.recv().await {
			let event_json = match serde_json::to_string(&event) {
				Ok(json) => json,
				Err(e) => {
					error!("Failed to serialize event: {}", e);
					continue;
				}
			};

			let event_cstring = CString::new(event_json).unwrap();
			let callback: extern "C" fn(*mut std::os::raw::c_void, *const std::os::raw::c_char) =
				unsafe { std::mem::transmute(callback_fn_ptr) };
			let callback_data_ptr: *mut std::os::raw::c_void =
				callback_data_int as *mut std::os::raw::c_void;
			callback(callback_data_ptr, event_cstring.as_ptr());
		}
	});
}

/// Start listening for core log messages
#[no_mangle]
pub extern "C" fn spawn_core_log_listener(
	callback: extern "C" fn(*mut std::os::raw::c_void, *const std::os::raw::c_char),
	callback_data: *mut std::os::raw::c_void,
) {
	let (Some(runtime), Some(core)) = (RUNTIME.get(), CORE.get()) else {
		error!("Core not initialized, cannot start log listener");
		return;
	};

	let callback_fn_ptr: usize = callback as usize;
	let callback_data_int: usize = callback_data as usize;

	let mut log_subscriber = core.logs.subscribe();

	// Never log from inside this loop: LogEventLayer would feed the message back into it.
	runtime.spawn(async move {
		while let Ok(log) = log_subscriber.recv().await {
			let log_json = match serde_json::to_string(&log) {
				Ok(json) => json,
				Err(_) => continue,
			};

			let log_cstring = CString::new(log_json).unwrap();
			let callback: extern "C" fn(*mut std::os::raw::c_void, *const std::os::raw::c_char) =
				unsafe { std::mem::transmute(callback_fn_ptr) };
			let callback_data_ptr: *mut std::os::raw::c_void =
				callback_data_int as *mut std::os::raw::c_void;
			callback(callback_data_ptr, log_cstring.as_ptr());
		}
	});
}

// Helper functions
// (send_response function removed - inlined into handle_core_msg)

async fn handle_json_rpc_request(request_json: String, core: &Arc<Core>) -> serde_json::Value {
	// Try parsing as batch first, then as single request
	let result: serde_json::Value = match serde_json::from_str::<Vec<JsonRpcRequest>>(&request_json)
	{
		Ok(batch) => {
			// Handle batch of requests
			let mut responses = Vec::new();
			for req in batch {
				responses.push(process_single_request(req, core).await);
			}
			serde_json::to_value(responses).unwrap()
		}
		Err(_) => {
			// Try as single request
			match serde_json::from_str::<JsonRpcRequest>(&request_json) {
				Ok(req) => {
					let response = process_single_request(req, core).await;
					serde_json::to_value(response).unwrap()
				}
				Err(e) => {
					serde_json::json!({
						"jsonrpc": "2.0",
						"id": "",
						"error": {
							"code": -32700,
							"message": format!("Parse error: {}", e)
						}
					})
				}
			}
		}
	};

	result
}

async fn process_single_request(
	jsonrpc_request: JsonRpcRequest,
	core: &Arc<Core>,
) -> JsonRpcResponse {
	let (daemon_request, request_id) = match convert_jsonrpc_to_daemon_request(&jsonrpc_request) {
		Ok(converted) => converted,
		Err(e) => {
			return JsonRpcResponse {
				jsonrpc: "2.0".to_string(),
				id: jsonrpc_request.id,
				result: None,
				error: Some(JsonRpcError {
					code: -32601,
					message: e,
				}),
			};
		}
	};

	let daemon_response = process_daemon_request(daemon_request, core).await;

	convert_daemon_response_to_jsonrpc(daemon_response, request_id)
}

fn convert_jsonrpc_to_daemon_request(
	jsonrpc: &JsonRpcRequest,
) -> Result<(DaemonRequest, String), String> {
	let library_id = if let Some(lib_id_str) = &jsonrpc.params.library_id {
		Some(Uuid::parse_str(lib_id_str).map_err(|e| format!("Invalid library ID: {}", e))?)
	} else {
		None
	};

	let daemon_request = if jsonrpc.method.starts_with("query:") {
		let payload = if jsonrpc.params.input.is_object()
			&& jsonrpc.params.input.as_object().unwrap().is_empty()
		{
			serde_json::Value::Null
		} else {
			jsonrpc.params.input.clone()
		};

		DaemonRequest::Query {
			method: jsonrpc.method.clone(),
			library_id,
			payload,
		}
	} else if jsonrpc.method.starts_with("action:") {
		DaemonRequest::Action {
			method: jsonrpc.method.clone(),
			library_id,
			payload: jsonrpc.params.input.clone(),
		}
	} else {
		return Err(format!("Invalid method prefix: {}", jsonrpc.method));
	};

	Ok((daemon_request, jsonrpc.id.clone()))
}

async fn process_daemon_request(request: DaemonRequest, core: &Arc<Core>) -> DaemonResponse {
	match request {
		DaemonRequest::Query {
			method,
			library_id,
			payload,
		} => match RpcServer::execute_json_operation(&method, library_id, payload, core).await {
			Ok(json_result) => DaemonResponse::JsonOk(json_result),
			Err(e) => DaemonResponse::Error(DaemonError::OperationFailed(e)),
		},
		DaemonRequest::Action {
			method,
			library_id,
			payload,
		} => match RpcServer::execute_json_operation(&method, library_id, payload, core).await {
			Ok(json_result) => DaemonResponse::JsonOk(json_result),
			Err(e) => DaemonResponse::Error(DaemonError::OperationFailed(e)),
		},
		_ => DaemonResponse::Error(DaemonError::OperationFailed(
			"Unsupported request type".to_string(),
		)),
	}
}

fn convert_daemon_response_to_jsonrpc(
	response: DaemonResponse,
	request_id: String,
) -> JsonRpcResponse {
	match response {
		DaemonResponse::JsonOk(json) => JsonRpcResponse {
			jsonrpc: "2.0".to_string(),
			id: request_id,
			result: Some(json),
			error: None,
		},
		DaemonResponse::Error(daemon_error) => JsonRpcResponse {
			jsonrpc: "2.0".to_string(),
			id: request_id,
			result: None,
			error: Some(JsonRpcError {
				code: -32603,
				message: daemon_error.to_string(),
			}),
		},
		_ => JsonRpcResponse {
			jsonrpc: "2.0".to_string(),
			id: request_id,
			result: None,
			error: Some(JsonRpcError {
				code: -32603,
				message: "Unsupported response type".to_string(),
			}),
		},
	}
}

// Android JNI bindings
#[cfg(target_os = "android")]
mod android {
	use super::*;
	use jni::{
		objects::{GlobalRef, JClass, JObject, JString, JValue},
		sys::jint,
		JNIEnv, JavaVM,
	};
	use once_cell::sync::OnceCell;
	use std::sync::Arc;

	static JAVA_VM: OnceCell<Arc<JavaVM>> = OnceCell::new();
	static EVENT_MODULE_REF: OnceCell<GlobalRef> = OnceCell::new();
	static LOG_MODULE_REF: OnceCell<GlobalRef> = OnceCell::new();

	// Only for Android x86_64 - provides missing symbol
	#[cfg(all(target_os = "android", target_arch = "x86_64"))]
	#[no_mangle]
	pub extern "C" fn __rust_probestack() {
		// Intentionally empty - stack probing disabled for x86_64 emulator
	}

	extern "C" {
		fn __android_log_write(
			prio: std::os::raw::c_int,
			tag: *const std::os::raw::c_char,
			text: *const std::os::raw::c_char,
		) -> std::os::raw::c_int;
	}

	const ANDROID_LOG_INFO: std::os::raw::c_int = 4;

	/// `tracing` writer that sends each formatted event to logcat under the `WingDriveCore` tag.
	#[derive(Default)]
	pub(super) struct LogcatWriter;

	impl std::io::Write for LogcatWriter {
		fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
			let line = String::from_utf8_lossy(buf);
			let text = CString::new(line.trim_end().replace('\0', " ")).unwrap_or_default();
			unsafe {
				__android_log_write(ANDROID_LOG_INFO, c"WingDriveCore".as_ptr(), text.as_ptr())
			};
			Ok(buf.len())
		}

		fn flush(&mut self) -> std::io::Result<()> {
			Ok(())
		}
	}

	/// Sends panic messages to logcat. A panic that reaches a JNI entry point aborts the
	/// process, and Android discards stderr, so without this the reason never shows up.
	fn log_panics_to_logcat() {
		const ANDROID_LOG_FATAL: std::os::raw::c_int = 7;
		std::panic::set_hook(Box::new(|info| {
			let text = CString::new(info.to_string().replace('\0', " ")).unwrap_or_default();
			unsafe {
				__android_log_write(ANDROID_LOG_FATAL, c"WingDriveCore".as_ptr(), text.as_ptr())
			};
		}));
	}

	/// Stores the JavaVM for callbacks and hands the application Context to `ndk-context`,
	/// which iroh's DNS resolver and `netdev` read; both panic when it is missing.
	fn init_android_context(env: &mut JNIEnv) -> jni::errors::Result<()> {
		let jvm = env.get_java_vm()?;
		let vm_ptr = jvm.get_java_vm_pointer();
		let _ = JAVA_VM.set(Arc::new(jvm));

		static APP_CONTEXT: OnceCell<GlobalRef> = OnceCell::new();
		if APP_CONTEXT.get().is_some() {
			return Ok(());
		}
		let app = env
			.call_static_method(
				"android/app/ActivityThread",
				"currentApplication",
				"()Landroid/app/Application;",
				&[],
			)?
			.l()?;
		let app_ref = env.new_global_ref(app)?;
		unsafe {
			ndk_context::initialize_android_context(
				vm_ptr.cast(),
				app_ref.as_obj().as_raw().cast(),
			);
		}
		let _ = APP_CONTEXT.set(app_ref);
		Ok(())
	}

	#[no_mangle]
	pub unsafe extern "C" fn Java_com_wingdrive_core_WingMobileCoreModule_initializeCore(
		mut env: JNIEnv,
		_class: JClass,
		data_dir: JString,
		device_name: JString,
	) -> jint {
		log_panics_to_logcat();
		super::init_tracing();
		if let Err(e) = init_android_context(&mut env) {
			tracing::error!("Failed to initialize the Android context: {e}");
			return -1;
		}

		let data_dir_str: String = match env.get_string(&data_dir) {
			Ok(s) => s.into(),
			Err(e) => {
				tracing::error!("Invalid data_dir string: {e}");
				return -1;
			}
		};
		let device_name_str: Option<String> = if device_name.is_null() {
			None
		} else {
			env.get_string(&device_name).ok().map(Into::into)
		};

		let (Ok(data_dir_cstr), Ok(device_name_cstr)) = (
			CString::new(data_dir_str),
			device_name_str.map(CString::new).transpose(),
		) else {
			tracing::error!("data_dir or device_name contains a NUL byte");
			return -1;
		};

		super::initialize_core(
			data_dir_cstr.as_ptr(),
			device_name_cstr
				.as_ref()
				.map(|s| s.as_ptr())
				.unwrap_or(std::ptr::null()),
		) as jint
	}

	#[no_mangle]
	pub unsafe extern "C" fn Java_com_wingdrive_core_WingMobileCoreModule_shutdownCore(
		_env: JNIEnv,
		_class: JClass,
	) {
		super::shutdown_core();
	}

	/// Calls a `(String) -> Unit` method on `target` from any thread. Native callback threads are
	/// attached once and stay attached, so the JVM does not attach and detach on every message.
	fn call_with_string(target: &JObject, method: &str, value: &str) -> jni::errors::Result<()> {
		let jvm = JAVA_VM.get().ok_or(jni::errors::Error::NullPtr("JavaVM"))?;
		let mut env = jvm.attach_current_thread_permanently()?;
		let jstring = env.new_string(value)?;
		env.call_method(
			target,
			method,
			"(Ljava/lang/String;)V",
			&[JValue::Object(&jstring)],
		)?;
		Ok(())
	}

	#[no_mangle]
	pub unsafe extern "C" fn Java_com_wingdrive_core_WingMobileCoreModule_handleCoreMsg(
		mut env: JNIEnv,
		_class: JClass,
		query: JString,
		promise: JObject,
	) {
		let query_cstr = match env.get_string(&query).map(String::from) {
			Ok(query) => CString::new(query).map_err(|_| "query contains a NUL byte".to_string()),
			Err(e) => Err(format!("invalid query string: {e}")),
		};
		let query_cstr = match query_cstr {
			Ok(query) => query,
			Err(message) => {
				if let Ok(message) = env.new_string(&message) {
					let _ = env.call_method(
						&promise,
						"reject",
						"(Ljava/lang/String;)V",
						&[JValue::Object(&message)],
					);
				}
				return;
			}
		};

		let promise_ref = match env.new_global_ref(promise) {
			Ok(promise_ref) => promise_ref,
			Err(e) => {
				tracing::error!("Failed to keep the promise reference: {e}");
				return;
			}
		};

		extern "C" fn android_callback(
			data: *mut std::os::raw::c_void,
			result: *const std::os::raw::c_char,
		) {
			let promise_ref = unsafe { Box::from_raw(data as *mut GlobalRef) };
			let result_str = unsafe { CStr::from_ptr(result).to_string_lossy() };
			if let Err(e) = call_with_string(promise_ref.as_obj(), "resolve", &result_str) {
				tracing::error!("Failed to resolve a core promise: {e}");
			}
		}

		let promise_ptr = Box::into_raw(Box::new(promise_ref)) as *mut std::os::raw::c_void;

		super::handle_core_msg(query_cstr.as_ptr(), android_callback, promise_ptr);
	}

	/// Stores the module reference, then starts `listener` with `callback`. Shared by the event and
	/// log listeners, which differ only in the reference slot and the Kotlin method they call.
	fn register_listener(
		env: &mut JNIEnv,
		module: JObject,
		slot: &OnceCell<GlobalRef>,
		callback: extern "C" fn(*mut std::os::raw::c_void, *const std::os::raw::c_char),
		listener: extern "C" fn(
			extern "C" fn(*mut std::os::raw::c_void, *const std::os::raw::c_char),
			*mut std::os::raw::c_void,
		),
	) {
		let registered = (|| -> jni::errors::Result<()> {
			let _ = JAVA_VM.set(Arc::new(env.get_java_vm()?));
			let _ = slot.set(env.new_global_ref(module)?);
			Ok(())
		})();
		match registered {
			Ok(()) => listener(callback, std::ptr::null_mut()),
			Err(e) => tracing::error!("Failed to register a core listener: {e}"),
		}
	}

	#[no_mangle]
	pub unsafe extern "C" fn Java_com_wingdrive_core_WingMobileCoreModule_registerCoreEventListener(
		mut env: JNIEnv,
		module: JObject,
	) {
		extern "C" fn android_event_callback(
			_data: *mut std::os::raw::c_void,
			event: *const std::os::raw::c_char,
		) {
			let event_str = unsafe { CStr::from_ptr(event).to_string_lossy() };
			if let Some(module_ref) = EVENT_MODULE_REF.get() {
				if let Err(e) = call_with_string(module_ref.as_obj(), "sendCoreEvent", &event_str) {
					tracing::error!("Failed to forward a core event: {e}");
				}
			}
		}

		register_listener(
			&mut env,
			module,
			&EVENT_MODULE_REF,
			android_event_callback,
			super::spawn_core_event_listener,
		);
	}

	#[no_mangle]
	pub unsafe extern "C" fn Java_com_wingdrive_core_WingMobileCoreModule_registerCoreLogListener(
		mut env: JNIEnv,
		module: JObject,
	) {
		extern "C" fn android_log_callback(
			_data: *mut std::os::raw::c_void,
			log: *const std::os::raw::c_char,
		) {
			let log_str = unsafe { CStr::from_ptr(log).to_string_lossy() };
			if let Some(module_ref) = LOG_MODULE_REF.get() {
				// No tracing here: a logged failure would come back through this same callback.
				let _ = call_with_string(module_ref.as_obj(), "sendCoreLog", &log_str);
			}
		}

		register_listener(
			&mut env,
			module,
			&LOG_MODULE_REF,
			android_log_callback,
			super::spawn_core_log_listener,
		);
	}
}
