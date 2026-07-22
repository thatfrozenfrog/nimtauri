import json, unittest
import dispatcher, protocol

suite "dispatcher":
  test "adds values":
    let request = RpcRequest(id: "1", methodName: "math.add", params: %*{"a": 2, "b": 3})
    let response = dispatch(request)
    check response["ok"].getBool
    check response["result"]["value"].getFloat == 5

  test "returns method not found":
    let request = RpcRequest(id: "2", methodName: "missing", params: newJObject())
    let response = dispatch(request)
    check not response["ok"].getBool
    check response["error"]["code"].getStr == "METHOD_NOT_FOUND"

  test "rejects non-numeric values for math.add":
    let request = RpcRequest(
      id: "3",
      methodName: "math.add",
      params: %*{"a": "not a number", "b": 3},
    )
    let response = dispatch(request)
    check not response["ok"].getBool
    check response["error"]["code"].getStr == "INVALID_PARAMS"
