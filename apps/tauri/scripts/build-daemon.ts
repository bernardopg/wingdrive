const target = process.env.TAURI_ENV_TARGET_TRIPLE || process.env.CARGO_BUILD_TARGET;
const command = ["cargo", "build", "--locked", "--manifest-path", "../../Cargo.toml", "--bin", "wing-daemon", "--features", "ffmpeg,heif"];
if (process.argv.includes("--release")) command.push("--release");
if (target) command.push("--target", target);
process.exit(await Bun.spawn(command, {stdout: "inherit", stderr: "inherit"}).exited);
