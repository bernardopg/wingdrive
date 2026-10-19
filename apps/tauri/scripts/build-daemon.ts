const target = process.env.TAURI_ENV_TARGET_TRIPLE || process.env.CARGO_BUILD_TARGET;
const release = process.argv.includes("--release");
// The CLI ships beside the daemon as a second sidecar.
const builds = [
	["--bin", "wing-daemon", "--features", "ffmpeg,heif"],
	["--bin", "wing"],
];
for (const args of builds) {
	const command = ["cargo", "build", "--locked", "--manifest-path", "../../Cargo.toml", ...args];
	if (release) command.push("--release");
	if (target) command.push("--target", target);
	const code = await Bun.spawn(command, {stdout: "inherit", stderr: "inherit"}).exited;
	if (code !== 0) process.exit(code);
}
