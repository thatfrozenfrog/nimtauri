import { execFileSync } from "node:child_process";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

const check = process.argv.includes("--check");
const root = join(import.meta.dir, "..");
const directories = ["backend-nim/src", "backend-nim/tests"];

async function nimFiles(directory: string): Promise<string[]> {
  const entries = await readdir(join(root, directory), { withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile() && entry.name.endsWith(".nim"))
    .map((entry) => join(root, directory, entry.name));
}

const files = (await Promise.all(directories.map(nimFiles))).flat();
const unformatted: string[] = [];

for (const file of files) {
  if (check) {
    const source = await readFile(file, "utf8");
    const formatted = execFileSync("nimpretty", ["--stdin"], {
      encoding: "utf8",
      input: source,
    });
    if (source !== formatted) unformatted.push(file);
  } else {
    execFileSync("nimpretty", [file], { stdio: "inherit" });
  }
}

if (unformatted.length > 0) {
  console.error("Nim files need formatting:\n" + unformatted.join("\n"));
  process.exit(1);
}
