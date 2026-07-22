# Tauri, Vue, and Nim

This project is a desktop app with three cooperating layers:

```text
Vue user interface
       │ invoke(...)
       ▼
Rust / Tauri bridge
       │ JSON lines over stdin/stdout
       ▼
Nim sidecar
```

Vue renders buttons, forms, and results. Nim performs backend work. Rust is the
small, trusted bridge between them. You normally add application features in
Vue and Nim; you only change Rust when the bridge itself needs new behavior.

## 1. What you need installed

- [Bun](https://bun.sh/)
- Rust stable, including the Windows C++ build tools on Windows
- Nim 2.x
- The Tauri prerequisites for your operating system

Install the JavaScript dependencies once:

```powershell
bun install
```

Start the development app:

```powershell
bun run tauri:dev
```

This starts Vite, builds the Nim sidecar, starts the Tauri desktop window, and
then starts the Nim process in the background.

### Supported platforms

The template currently supports native x64 builds for Windows and Linux:

- Windows: `x86_64-pc-windows-msvc`
- Linux: `x86_64-unknown-linux-gnu`

macOS and ARM targets are intentionally rejected by the Nim sidecar build
script. Build on the target operating system (or its GitHub Actions runner),
not by cross-compiling the sidecar from Windows.

## 2. Useful commands

| Command                                           | What it does                                       |
| ------------------------------------------------- | -------------------------------------------------- |
| `bun run tauri:dev`                               | Run the desktop app for development.               |
| `bun run typecheck`                               | Check Vue and TypeScript types.                    |
| `bun run test`                                    | Run the Vue/Vitest tests.                          |
| `bun run test:nim`                                | Compile and run Nim tests.                         |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Run Rust bridge tests.                             |
| `bun run build:sidecar`                           | Build just the Nim backend.                        |
| `bun run test:sidecar`                            | Send a real request to the compiled Nim backend.   |
| `bun run format`                                  | Format Vue/TypeScript, Rust, and Nim source files. |
| `bun run format:check`                            | Check formatting without changing files.           |
| `bun clean`                                       | Remove generated build output and caches.          |
| `bun run build:release`                           | Build installers and release bundles.              |

Use `bun run <name>` for package scripts. `bun build` is Bun's JavaScript
bundler and is not the project release command.

## 3. Where things live

```text
src/                         Vue application
  api/backend.ts             Typed functions that Vue uses to call Nim
  api/backend.types.ts       TypeScript request/result types
  views/ and components/     User interface

src-tauri/                   Tauri/Rust application shell
  src/lib.rs                 Starts and registers Rust commands
  src/sidecar.rs             Starts Nim and routes requests/responses
  tauri.conf.json            Tauri, bundle, and sidecar configuration

backend-nim/                 Nim backend
  src/main.nim               Reads requests and writes responses/events
  src/protocol.nim           JSON request/response helpers
  src/dispatcher.nim         Maps a method name to Nim code
  tests/                     Nim tests

scripts/build-sidecar.ts     Compiles Nim with the correct Tauri filename
```

## 4. The two kinds of messages

There are two ways Nim can communicate with Vue.

| Need                            | Use                  | Example                            |
| ------------------------------- | -------------------- | ---------------------------------- |
| Vue asks Nim for one answer     | Request and response | `math.add` returns a number.       |
| Nim wants to announce something | Event                | `backend.ready` announces startup. |

The important rule is: **stdout is protocol-only**. Nim writes JSON messages to
stdout, one JSON object per line. Write debugging logs to stderr instead.

The first event must be `backend.ready` with protocol version `1`. Rust rejects
and stops a sidecar with another version. This prevents an old backend binary
from silently talking a different protocol than the desktop app expects.

Bad:

```nim
echo "starting calculation" # Breaks the JSON protocol.
```

Good:

```nim
stderr.writeLine("starting calculation")
stderr.flushFile()
```

## 5. Calling Nim from Vue

The simplest path is a request and response. The existing `addNumbers` helper
is a complete example:

```ts
// src/api/backend.ts
export const addNumbers = (a: number, b: number) =>
  callBackend<{ a: number; b: number }, AddResult>("math.add", { a, b });
```

From a Vue component, call it with `await`:

```vue
<script setup lang="ts">
import { ref } from "vue";
import { addNumbers } from "@/api/backend";

const first = ref(2);
const second = ref(3);
const result = ref<number | null>(null);
const error = ref("");

async function add() {
  error.value = "";
  try {
    const response = await addNumbers(first.value, second.value);
    result.value = response.value;
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : String(reason);
  }
}
</script>

<template>
  <button @click="add">Add</button>
  <p v-if="result !== null">Answer: {{ result }}</p>
  <p v-if="error">{{ error }}</p>
</template>
```

`addNumbers` returns a JavaScript `Promise`. `await` pauses only the `add`
function until Nim replies; it does not freeze the app window.

## 6. What happens behind `addNumbers`

Here is the actual trip taken by a request:

1. Vue calls `addNumbers(2, 3)`.
2. `src/api/backend.ts` calls Tauri's `invoke("backend_call", ...)` API.
3. Rust receives the `backend_call` command in `src-tauri/src/sidecar.rs`.
4. Rust assigns a unique request ID and writes this one-line JSON message to
   Nim's standard input:

   ```json
   { "id": "a-unique-id", "method": "math.add", "params": { "a": 2, "b": 3 } }
   ```

5. `backend-nim/src/main.nim` reads the line and calls `dispatch`.
6. `backend-nim/src/dispatcher.nim` finds the `"math.add"` branch.
7. Nim writes a response such as this to standard output:

   ```json
   { "id": "a-unique-id", "ok": true, "result": { "value": 5.0 } }
   ```

8. Rust matches the response ID to the waiting Vue request and resolves the
   Promise with `{ value: 5 }`.

Rust keeps the IDs because many Vue requests may be in flight at the same time.
Do not need to create IDs yourself in Vue; the Rust bridge does that safely.

Rust waits up to 15 seconds for a normal request. It returns a typed
`REQUEST_TIMEOUT` error if Nim does not reply in time.

## 7. Add your own Nim method: a complete example

This example adds a method named `text.shout`. It takes a message and returns
the message in uppercase. Make the changes in this order.

### Step A: Write the Nim behavior

In `backend-nim/src/dispatcher.nim`, add `strutils` to the imports:

```nim
import json, times, strutils
```

Then add this branch inside the `case request.methodName` statement:

```nim
of "text.shout":
  requireObject(request.params)
  if not request.params.hasKey("message") or
      request.params["message"].kind != JString:
    return errorResponse(request.id, "INVALID_PARAMS", "message must be a string")

  let message = request.params["message"].getStr
  return successResponse(request.id, %*{"value": message.toUpperAscii})
```

The method name must exactly match the TypeScript method name you add next.

### Step B: Describe the response in TypeScript

In `src/api/backend.types.ts`:

```ts
export interface ShoutResult {
  value: string;
}
```

### Step C: Create a small, typed Vue-facing helper

In `src/api/backend.ts`, import `ShoutResult`, then add:

```ts
export const shout = (message: string) =>
  callBackend<{ message: string }, ShoutResult>("text.shout", { message });
```

Prefer helpers such as `shout()` over calling `callBackend()` directly from
every component. One helper gives you a single place to improve types or adjust
the backend method later.

### Step D: Call it from Vue

```vue
<script setup lang="ts">
import { ref } from "vue";
import { shout } from "@/api/backend";

const message = ref("");
const answer = ref("");
const error = ref("");

async function makeUppercase() {
  error.value = "";
  try {
    answer.value = (await shout(message.value)).value;
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : String(reason);
  }
}
</script>

<template>
  <input v-model="message" placeholder="Type something" />
  <button @click="makeUppercase">Shout</button>
  <p>{{ answer }}</p>
  <p v-if="error">{{ error }}</p>
</template>
```

### Step E: Test it

Add a Nim test in `backend-nim/tests/test_dispatcher.nim` and run:

```powershell
bun run test:nim
bun run test
bun run typecheck
bun run format:check
bun run tauri:dev
```

You do not need to edit Rust for ordinary new Nim methods. Rust already forwards
the method string and JSON parameters through the `backend_call` bridge.

## 8. Receiving an event from Nim in Vue

Responses answer one request. Events are for information Nim sends without a
specific Vue request waiting for it.

Nim sends the existing startup event like this:

```nim
writeProtocol(eventMessage("backend.ready", %*{
  "protocolVersion": ProtocolVersion,
  "backendVersion": BackendVersion
}))
```

Rust forwards every Nim event to the frontend under the Tauri event name
`nim://event`. Listen for it in Vue like this:

```ts
import { onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

let unlisten: UnlistenFn | undefined;

onMounted(async () => {
  unlisten = await listen<{ event: string; data: unknown }>("nim://event", (message) => {
    if (message.payload.event === "backend.ready") {
      console.log("Nim is ready:", message.payload.data);
    }
  });
});

onUnmounted(() => {
  unlisten?.();
});
```

To publish a new event, make Nim write a valid event message:

```nim
writeProtocol(eventMessage("task.progress", %*{
  "completed": 4,
  "total": 10
}))
```

Use events for progress, notifications, or state changes. Use request/response
for actions such as calculating a result or saving a record.

## 9. Validation and error handling

Never trust values just because they came from your Vue interface. Validate them
in Nim before using them.

The dispatcher already follows this pattern:

```nim
requireObject(request.params)
if not request.params.hasKey("message") or
    request.params["message"].kind != JString:
  return errorResponse(request.id, "INVALID_PARAMS", "message must be a string")
```

Useful response forms are:

```json
{ "id": "1", "ok": true, "result": { "value": 5 } }
```

```json
{
  "id": "1",
  "ok": false,
  "error": { "code": "INVALID_PARAMS", "message": "message must be a string" }
}
```

In Vue, surround backend calls with `try`/`catch`. A backend error, a timeout,
or a stopped sidecar rejects the Promise.

The helper converts these failures into a typed `BackendError`, so you can show
helpful UI instead of parsing error text:

```ts
import { BackendError } from "@/api/backend.types";

try {
  await addNumbers(2, 3);
} catch (reason) {
  if (reason instanceof BackendError && reason.code === "INVALID_PARAMS") {
    console.log(reason.message);
  }
}
```

Common error codes are:

| Code                        | Meaning                                           |
| --------------------------- | ------------------------------------------------- |
| `INVALID_PARAMS`            | Nim rejected the supplied input.                  |
| `METHOD_NOT_FOUND`          | Vue called a method that Nim does not implement.  |
| `BACKEND_NOT_READY`         | The sidecar has not finished starting.            |
| `REQUEST_TIMEOUT`           | Nim did not answer within 15 seconds.             |
| `PROCESS_TERMINATED`        | The sidecar stopped before it could respond.      |
| `PROTOCOL_VERSION_MISMATCH` | The running sidecar is incompatible with the app. |

## 10. When should you edit Rust?

For normal app features, you usually do **not** edit Rust. The generic
`backend_call` command already sends a method and JSON parameters to Nim.

Edit Rust when you need one of these:

- Start, restart, or supervise the Nim process differently.
- Expose a new native Tauri capability that should not go through Nim.
- Add a carefully scoped plugin such as dialogs, filesystem access, or an
  updater.
- Change the timeout, event routing, or sidecar lifecycle.

Keep the Rust bridge narrow. Do not let the Vue webview run arbitrary shell
commands. In this project only Rust launches the configured Nim sidecar.

When the application closes, Rust first asks Nim to run `system.shutdown` and
waits briefly for its response. It only force-stops the process if it does not
finish. Put Nim cleanup work—such as flushing a file or closing a database—in
the `system.shutdown` branch when your app needs it.

## 11. Debugging checklist

| Symptom                                       | First thing to check                                                                               |
| --------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `BACKEND_NOT_READY`                           | Wait for the sidecar to start; use `getBackendStatus()`.                                           |
| Request times out                             | Check that Nim returns exactly one JSON response with the same ID.                                 |
| Invalid JSON error                            | Ensure stdout contains only one-line JSON; move logs to stderr.                                    |
| Method not found                              | Check that the string in `backend.ts` and the Nim `case` branch match.                             |
| `PROTOCOL_VERSION_MISMATCH`                   | Rebuild the sidecar; the template currently requires protocol version `1`.                         |
| `BackendError` in Vue                         | Use `error.code` to decide whether to show validation, timeout, or restart guidance.               |
| TypeScript error                              | Update `backend.types.ts` and run `bun run typecheck`.                                             |
| Nim import red lines                          | Reload the Nim language server; `backend-nim/nim.cfg` provides the source path.                    |
| The sidecar does not start in a release build | Run `bun run build:sidecar:release` and check its target-suffixed output in `src-tauri/binaries/`. |

For a direct backend check, run:

```powershell
bun run build:sidecar
bun run test:sidecar
```

## 12. A safe daily workflow

1. Add or change a Nim method in `dispatcher.nim`.
2. Add a Nim test for the method.
3. Add TypeScript request/result types.
4. Add a typed helper in `src/api/backend.ts`.
5. Call the helper from a Vue component or Pinia store.
6. Run `bun run test`, `bun run test:nim`, and `bun run typecheck`.
7. Run `bun run format:check` before committing.
8. Run `bun run tauri:dev` and try the feature in the desktop app.
9. Before sharing a build, run `bun run build:release`.

That is the core loop for this project: **define a small JSON contract, validate
it in Nim, expose a typed Vue helper, and test both sides.**

## 13. What CI checks for you

Every pull request and push to `main` runs the same core checks on Windows and
Ubuntu: formatting, TypeScript type checking, Vue tests, Nim tests, Rust bridge
tests, a compiled-sidecar smoke test, and a Tauri bundle build. A green CI run
means both supported operating systems have compiled the template successfully.
