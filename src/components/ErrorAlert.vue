<script setup lang="ts">
defineProps<{ message: string }>();
</script>

<template>
  <div role="alert" class="rounded-xl border border-red-900 bg-red-950/50 p-4 text-red-200">
    {{ message }}
  </div>
</template>
