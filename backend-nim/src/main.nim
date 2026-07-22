import json
import dispatcher, protocol

proc writeProtocol(node: JsonNode) =
  stdout.writeLine($node)
  stdout.flushFile()

writeProtocol(eventMessage("backend.ready", %*{
  "protocolVersion": ProtocolVersion,
  "backendVersion": BackendVersion
}))

while true:
  var line: string
  if not stdin.readLine(line):
    break
  if line.len == 0:
    continue

  var requestId = ""
  try:
    let request = parseRequest(line)
    requestId = request.id
    let response = dispatch(request)
    writeProtocol(response)
    if request.methodName == "system.shutdown":
      break
  except JsonParsingError as error:
    writeProtocol(errorResponse(requestId, "INVALID_REQUEST", error.msg))
  except ValueError as error:
    writeProtocol(errorResponse(requestId, "INVALID_REQUEST", error.msg))
  except CatchableError as error:
    stderr.writeLine("Unhandled backend error: " & error.msg)
    stderr.flushFile()
    writeProtocol(errorResponse(requestId, "INTERNAL_ERROR",
        "unexpected backend error"))
