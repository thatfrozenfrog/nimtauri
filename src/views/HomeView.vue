<template>
  <section class="space-y-8">
    <div class="space-y-4">
      <p class="text-sm font-medium uppercase tracking-[0.2em] text-emerald-400">Desktop starter</p>
      <h1 class="max-w-3xl text-4xl font-bold tracking-tight sm:text-6xl">
        Vue on the surface. Nim under the hood.
      </h1>
      <p class="max-w-2xl text-lg leading-8 text-zinc-400">
        A Tauri v2 template with Vue Router, Pinia, Tailwind CSS, Bun, and a persistent Nim sidecar
        connected through newline-delimited JSON.
      </p>
    </div>

    <div class="grid gap-4 sm:grid-cols-2">
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900 p-6">
        <h2 class="font-semibold">Frontend</h2>
        <p class="mt-2 text-zinc-400">Vue 3, Vite, Router, Pinia, Tailwind, native fetch.</p>
      </div>
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900 p-6">
        <h2 class="font-semibold">Backend</h2>
        <p class="mt-2 text-zinc-400">Persistent Nim process managed by a narrow Rust bridge.</p>
      </div>
    </div>

    <RouterLink
      to="/backend"
      class="inline-flex rounded-lg bg-white px-4 py-2 font-medium text-zinc-950 hover:bg-zinc-200"
    >
      Open backend demo
    </RouterLink>
  </section>
</template>
