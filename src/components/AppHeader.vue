<template>
  <header class="border-b border-zinc-800 bg-zinc-950/90 backdrop-blur">
    <div class="mx-auto flex max-w-5xl items-center justify-between px-5 py-4">
      <RouterLink to="/" class="font-semibold tracking-tight"> Tauri + Vue + Nim </RouterLink>
      <nav class="flex gap-5 text-sm text-zinc-400">
        <RouterLink class="hover:text-white" to="/">Home</RouterLink>
        <RouterLink class="hover:text-white" to="/backend">Backend demo</RouterLink>
      </nav>
    </div>
  </header>
</template>
