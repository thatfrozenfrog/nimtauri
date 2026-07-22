version       = "0.1.0"
author        = "thatfrozenfrog"
description   = "Nim JSON Lines sidecar backend for Tauri"
license       = "MIT"
srcDir        = "src"
bin           = @["main"]

requires "nim >= 2.0.0"

task test, "Run backend tests":
  exec "nim c -r --nimcache:nimcache/test_protocol --path:src tests/test_protocol.nim"
  exec "nim c -r --nimcache:nimcache/test_dispatcher --path:src tests/test_dispatcher.nim"
