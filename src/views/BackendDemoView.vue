<script setup lang="ts">
import { computed, ref } from "vue";
import { addNumbers, getSystemInfo, ping } from "@/api/backend";
import type { SystemInfo } from "@/api/backend.types";
import { sendBackendNotification } from "@/api/notification";
import BackendStatus from "@/components/BackendStatus.vue";
import ErrorAlert from "@/components/ErrorAlert.vue";
import { useBackendStore } from "@/stores/backend";

const backend = useBackendStore();
const message = ref("Hello from Vue");
const pingOutput = ref("");
const a = ref(2);
const b = ref(3);
const sum = ref<number | null>(null);
const info = ref<SystemInfo | null>(null);
const error = ref("");
const notificationMessage = ref("");
const busy = ref(false);
const isReady = computed(() => backend.status.state === "ready");

async function run<T>(operation: () => Promise<T>): Promise<T | undefined> {
  busy.value = true;
  error.value = "";
  try {
    return await operation();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    busy.value = false;
    await backend.refreshStatus();
  }
}

async function runPing() {
  const result = await run(() => ping(message.value));
  if (result) pingOutput.value = `${result.message} @ ${result.timestamp}`;
}

async function runAdd() {
  const result = await run(() => addNumbers(a.value, b.value));
  if (result) sum.value = result.value;
}

async function loadInfo() {
  const result = await run(getSystemInfo);
  if (result) info.value = result;
}

async function notifyUser() {
  const sent = await run(sendBackendNotification);
  if (sent === true) notificationMessage.value = "Notification sent.";
  if (sent === false) notificationMessage.value = "Notification permission was not granted.";
}
</script>

<template>
  <section class="space-y-6">
    <div>
      <h1 class="text-3xl font-bold">Nim backend demo</h1>
      <p class="mt-2 text-zinc-400">Calls travel through Tauri IPC to a persistent Nim process.</p>
    </div>

    <BackendStatus :status="backend.status" />
    <ErrorAlert
      v-if="error || backend.status.lastError"
      :message="error || backend.status.lastError!"
    />

    <div class="grid gap-5 lg:grid-cols-2">
      <form class="rounded-2xl border border-zinc-800 bg-zinc-900 p-5" @submit.prevent="runPing">
        <h2 class="font-semibold">Ping</h2>
        <input
          v-model="message"
          class="mt-4 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 outline-none focus:border-emerald-400"
        />
        <button
          class="mt-3 rounded-lg bg-emerald-400 px-4 py-2 font-medium text-zinc-950 disabled:opacity-40"
          :disabled="busy || !isReady"
        >
          Send
        </button>
        <p v-if="pingOutput" class="mt-4 break-all text-sm text-zinc-300">{{ pingOutput }}</p>
      </form>

      <form class="rounded-2xl border border-zinc-800 bg-zinc-900 p-5" @submit.prevent="runAdd">
        <h2 class="font-semibold">Add numbers</h2>
        <div class="mt-4 flex gap-3">
          <input
            v-model.number="a"
            type="number"
            class="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2"
          />
          <input
            v-model.number="b"
            type="number"
            class="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2"
          />
        </div>
        <button
          class="mt-3 rounded-lg bg-white px-4 py-2 font-medium text-zinc-950 disabled:opacity-40"
          :disabled="busy || !isReady"
        >
          Add
        </button>
        <p v-if="sum !== null" class="mt-4 text-2xl font-bold">{{ sum }}</p>
      </form>
    </div>

    <div class="rounded-2xl border border-zinc-800 bg-zinc-900 p-5">
      <div class="flex flex-wrap gap-3">
        <button
          class="rounded-lg border border-zinc-700 px-4 py-2 hover:bg-zinc-800 disabled:opacity-40"
          :disabled="busy || !isReady"
          @click="loadInfo"
        >
          Load system info
        </button>
        <button
          class="rounded-lg border border-zinc-700 px-4 py-2 hover:bg-zinc-800 disabled:opacity-40"
          :disabled="busy"
          @click="notifyUser"
        >
          Send notification
        </button>
        <button
          class="rounded-lg border border-zinc-700 px-4 py-2 hover:bg-zinc-800 disabled:opacity-40"
          :disabled="busy"
          @click="backend.restart"
        >
          Restart backend
        </button>
      </div>
      <p v-if="notificationMessage" class="mt-4 text-sm text-zinc-300">
        {{ notificationMessage }}
      </p>
      <pre v-if="info" class="mt-4 overflow-auto rounded-lg bg-zinc-950 p-4 text-sm">{{
        JSON.stringify(info, null, 2)
      }}</pre>
    </div>
  </section>
</template>
