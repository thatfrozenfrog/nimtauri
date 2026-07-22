import json, times
import protocol

proc requireObject(params: JsonNode) =
  if params.kind != JObject:
    raise newException(ValueError, "params must be an object")

proc dispatch*(request: RpcRequest): JsonNode =
  try:
    case request.methodName
    of "ping":
      requireObject(request.params)
      if not request.params.hasKey("message") or request.params[
          "message"].kind != JString:
        return errorResponse(request.id, "INVALID_PARAMS", "message must be a string")
      return successResponse(request.id, %*{
        "message": request.params["message"].getStr,
        "timestamp": getTime().toUnix
      })

    of "math.add":
      requireObject(request.params)
      if not request.params.hasKey("a") or not request.params.hasKey("b"):
        return errorResponse(request.id, "INVALID_PARAMS", "a and b are required")
      if request.params["a"].kind notin {JInt, JFloat} or
          request.params["b"].kind notin {JInt, JFloat}:
        return errorResponse(request.id, "INVALID_PARAMS", "a and b must be numbers")
      let a = request.params["a"].getFloat
      let b = request.params["b"].getFloat
      return successResponse(request.id, %*{"value": a + b})

    of "system.info":
      return successResponse(request.id, %*{
        "backend": "nim",
        "backendVersion": BackendVersion,
        "nimVersion": NimVersion,
        "protocolVersion": ProtocolVersion,
        "operatingSystem": hostOS,
        "architecture": hostCPU
      })

    of "system.shutdown":
      return successResponse(request.id, %*{"shuttingDown": true})

    else:
      return errorResponse(request.id, "METHOD_NOT_FOUND", "unknown method: " &
          request.methodName)
  except ValueError as error:
    return errorResponse(request.id, "INVALID_PARAMS", error.msg)
  except CatchableError:
    return errorResponse(request.id, "INTERNAL_ERROR", "unexpected backend error")
