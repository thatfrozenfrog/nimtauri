<script setup lang="ts">
import { onMounted } from "vue";
import AppHeader from "@/components/AppHeader.vue";
import { useBackendStore } from "@/stores/backend";

const backend = useBackendStore();

onMounted(() => {
  void backend.initialize();
});
</script>

<template>
  <div class="min-h-screen bg-zinc-950 text-zinc-100">
    <AppHeader />
    <main class="mx-auto max-w-5xl px-5 py-10">
      <RouterView />
    </main>
  </div>
</template>
