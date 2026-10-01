import React from "react";
import ReactDOM from "react-dom/client";
import { PlatformProvider, Shell } from "@wingdrive/interface";
import { SpacedriveClient, HttpTransport } from "@wingdrive/ts-client";
import { platform } from "./platform";
import "./index.css";
import "@wingdrive/interface/styles.css";

// Talk to wing-server's /rpc endpoint on the same origin the page was loaded from.
// This works both standalone (browser → wing-server) and embedded inside an iframe.
const client = new SpacedriveClient(new HttpTransport());

function App() {
	return (
		<PlatformProvider platform={platform}>
			<Shell client={client} />
		</PlatformProvider>
	);
}

ReactDOM.createRoot(document.getElementById("root")!).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>
);