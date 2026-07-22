import json, unittest
import protocol

suite "protocol":
  test "parses a valid request":
    let request = parseRequest("""{"id":"1","method":"ping","params":{"message":"hi"}}""")
    check request.id == "1"
    check request.methodName == "ping"
    check request.params["message"].getStr == "hi"

  test "rejects a missing id":
    expect ValueError:
      discard parseRequest("""{"method":"ping"}""")

  test "builds a one-line response":
    let response = $successResponse("1", %*{"ok": "yes"})
    check '\n' notin response
