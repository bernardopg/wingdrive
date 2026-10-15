import React from "react";
import ReactDOM from "react-dom/client";
import { AuxiliaryWindow, PlatformProvider, Settings, Shell } from "@wingdrive/interface";
import { WingDriveClient, HttpTransport } from "@wingdrive/ts-client";
import { platform } from "./platform";
import "./index.css";
import "@wingdrive/interface/styles.css";

// Talk to wing-server's /rpc endpoint on the same origin the page was loaded from.
// This works both standalone (browser → wing-server) and embedded inside an iframe.
const client = new WingDriveClient(new HttpTransport());

// The desktop app keeps the active library in the native layer; the browser
// keeps it here so a reload stays in the same library.
const LIBRARY_KEY = "wingdrive-current-library";
try {
	const saved = localStorage.getItem(LIBRARY_KEY);
	if (saved) client.setCurrentLibrary(saved, false);
} catch {}
client.on("library-changed", (id: string) => {
	try {
		localStorage.setItem(LIBRARY_KEY, id);
	} catch {}
});

function App() {
	// Settings opens in its own tab, the browser counterpart of the desktop Settings window
	const isSettings = window.location.pathname.startsWith("/settings");
	return (
		<PlatformProvider platform={platform}>
			{isSettings ? (
				<AuxiliaryWindow client={client}>
					<Settings />
				</AuxiliaryWindow>
			) : (
				<Shell client={client} />
			)}
		</PlatformProvider>
	);
}

ReactDOM.createRoot(document.getElementById("root")!).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>
);