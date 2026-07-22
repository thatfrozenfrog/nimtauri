const debug = process.env.TAURI_ENV_DEBUG === "true";
const sidecarArgs = debug ? [] : ["--release"];

function run(command: string[]) {
  const result = Bun.spawnSync(command, {
    stdout: "inherit",
    stderr: "inherit",
  });

  if (result.exitCode !== 0) {
    throw new Error(`${command.join(" ")} failed with exit code ${result.exitCode}`);
  }
}

run(["bun", "scripts/build-sidecar.ts", ...sidecarArgs]);
run(["bun", "run", "build:web"]);
