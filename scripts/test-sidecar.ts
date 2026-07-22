import { join } from "node:path";

const target = (await Bun.$`rustc --print host-tuple`.text()).trim();
const extension = target.includes("windows") ? ".exe" : "";
const binary = join(
  import.meta.dir,
  "..",
  "src-tauri",
  "binaries",
  `nim-backend-${target}${extension}`,
);

const child = Bun.spawn([binary], {
  stdin: "pipe",
  stdout: "pipe",
  stderr: "inherit",
});

const reader = child.stdout.getReader();
const decoder = new TextDecoder();
let buffer = "";

async function nextMessage(): Promise<Record<string, unknown>> {
  while (true) {
    const newline = buffer.indexOf("\n");
    if (newline >= 0) {
      const line = buffer.slice(0, newline).trim();
      buffer = buffer.slice(newline + 1);
      if (line) return JSON.parse(line);
    }

    const chunk = await reader.read();
    if (chunk.done) throw new Error("Sidecar exited before returning a message");
    buffer += decoder.decode(chunk.value, { stream: true });
  }
}

const ready = await nextMessage();
if (ready.event !== "backend.ready") throw new Error("Missing backend.ready event");

child.stdin.write(
  JSON.stringify({
    id: "smoke-1",
    method: "ping",
    params: { message: "smoke" },
  }) + "\n",
);
child.stdin.flush();

const pong = await nextMessage();
if (pong.id !== "smoke-1" || pong.ok !== true) throw new Error("Invalid ping response");

child.stdin.write(
  JSON.stringify({
    id: "smoke-2",
    method: "system.shutdown",
    params: {},
  }) + "\n",
);
child.stdin.flush();

const shutdown = await nextMessage();
if (shutdown.id !== "smoke-2" || shutdown.ok !== true) throw new Error("Invalid shutdown response");

child.stdin.end();
const exitCode = await child.exited;
if (exitCode !== 0) throw new Error(`Sidecar exited with ${exitCode}`);

console.log("Sidecar smoke test passed");
