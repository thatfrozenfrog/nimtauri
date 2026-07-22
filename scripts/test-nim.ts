const tests = ["test_protocol.nim", "test_dispatcher.nim"];

for (const test of tests) {
  const cacheName = test.replace(/\.nim$/, "");
  const result = Bun.spawnSync(
    [
      "nim",
      "c",
      "-r",
      `--nimcache:backend-nim/nimcache/${cacheName}`,
      "--path:backend-nim/src",
      `backend-nim/tests/${test}`,
    ],
    { stdout: "inherit", stderr: "inherit" },
  );
  if (result.exitCode !== 0) process.exit(result.exitCode);
}
