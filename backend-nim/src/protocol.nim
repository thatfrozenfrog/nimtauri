import json

const
  ProtocolVersion* = 1
  BackendVersion* = "0.1.0"
  MaxMessageBytes* = 1024 * 1024

type
  RpcRequest* = object
    id*: string
    methodName*: string
    params*: JsonNode

proc parseRequest*(line: string): RpcRequest =
  if line.len > MaxMessageBytes:
    raise newException(ValueError, "request exceeds maximum message size")

  let node = parseJson(line)
  if node.kind != JObject:
    raise newException(ValueError, "request must be a JSON object")
  if not node.hasKey("id") or node["id"].kind != JString or node[
      "id"].getStr.len == 0:
    raise newException(ValueError, "request id must be a non-empty string")
  if not node.hasKey("method") or node["method"].kind != JString or node[
      "method"].getStr.len == 0:
    raise newException(ValueError, "method must be a non-empty string")

  result.id = node["id"].getStr
  result.methodName = node["method"].getStr
  result.params = if node.hasKey("params"): node["params"] else: newJObject()

proc successResponse*(id: string, value: JsonNode): JsonNode =
  %*{"id": id, "ok": true, "result": value}

proc errorResponse*(id, code, message: string): JsonNode =
  %*{"id": id, "ok": false, "error": {"code": code, "message": message}}

proc eventMessage*(name: string, data: JsonNode): JsonNode =
  %*{"event": name, "data": data}
