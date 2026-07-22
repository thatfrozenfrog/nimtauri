import { lstat, readdir, rm } from "node:fs/promises";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
const directories = [
  "dist",
  "src-tauri/target",
  "src-tauri/gen",
  "backend-nim/nimcache",
  "node_modules/.vite",
];
const files = [
  "backend-nim/src/main",
  "backend-nim/src/main.exe",
  "backend-nim/tests/test_protocol",
  "backend-nim/tests/test_protocol.exe",
  "backend-nim/tests/test_dispatcher",
  "backend-nim/tests/test_dispatcher.exe",
];

async function remove(path: string) {
  const target = join(root, path);
  const exists = await lstat(target)
    .then(() => true)
    .catch(() => false);
  if (!exists) return;

  await rm(target, {
    recursive: true,
    force: true,
    maxRetries: 3,
    retryDelay: 200,
  });
  console.log(`Removed ${path}`);
}

await Promise.all([...directories, ...files].map(remove));

const binariesDirectory = join(root, "src-tauri", "binaries");
const binaries = await readdir(binariesDirectory, { withFileTypes: true }).catch(() => []);
await Promise.all(
  binaries
    .filter((entry) => entry.isFile() && entry.name.startsWith("nim-backend-"))
    .map((entry) => remove(`src-tauri/binaries/${entry.name}`)),
);

console.log("Clean complete.");
