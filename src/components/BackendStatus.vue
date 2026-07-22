<script setup lang="ts">
import { computed } from "vue";
import type { BackendStatus } from "@/api/backend.types";

const props = defineProps<{ status: BackendStatus }>();

const dotClass = computed(() => ({
  "bg-amber-400": props.status.state === "starting",
  "bg-emerald-400": props.status.state === "ready",
  "bg-zinc-500": props.status.state === "stopped",
  "bg-red-400": props.status.state === "failed",
}));
</script>

<template>
  <div class="flex items-center gap-3 rounded-xl border border-zinc-800 bg-zinc-900 p-4">
    <span class="h-2.5 w-2.5 rounded-full" :class="dotClass" />
    <div>
      <p class="font-medium capitalize">{{ status.state }}</p>
      <p class="text-sm text-zinc-400">
        Nim {{ status.version ?? "unknown" }} · protocol {{ status.protocol ?? "unknown" }}
      </p>
    </div>
  </div>
</template>
