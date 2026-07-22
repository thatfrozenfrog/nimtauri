import { chmod, mkdir } from "node:fs/promises";
import { join } from "node:path";

const release = process.argv.includes("--release");
const targetFlag = process.argv.find((value) => value.startsWith("--target="));
const target =
  targetFlag?.slice("--target=".length) ??
  process.env.TAURI_ENV_TARGET_TRIPLE ??
  (await Bun.$`rustc --print host-tuple`.text()).trim();

const supported = new Set(["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]);

if (!supported.has(target)) {
  throw new Error(`Unsupported target: ${target}`);
}

const extension = target.includes("windows") ? ".exe" : "";
const outputDir = join(import.meta.dir, "..", "src-tauri", "binaries");
const output = join(outputDir, `nim-backend-${target}${extension}`);
const source = join(import.meta.dir, "..", "backend-nim", "src", "main.nim");
const nimcache = join(import.meta.dir, "..", "backend-nim", "nimcache", target);

await mkdir(outputDir, { recursive: true });

const args = [
  "c",
  "--mm:orc",
  `--nimcache:${nimcache}`,
  `--out:${output}`,
  ...(release ? ["-d:release", "--opt:speed"] : []),
  source,
];

console.log(`Building Nim sidecar for ${target}`);
const processResult = Bun.spawnSync(["nim", ...args], {
  cwd: join(import.meta.dir, ".."),
  stdout: "inherit",
  stderr: "inherit",
});

if (processResult.exitCode !== 0) {
  throw new Error(`Nim compilation failed with exit code ${processResult.exitCode}`);
}

if (!extension) {
  await chmod(output, 0o755);
}

console.log(`Created ${output}`);
