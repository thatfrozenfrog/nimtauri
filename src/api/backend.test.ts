import { describe, expect, it } from "vitest";
import { toBackendError } from "./backend";

describe("toBackendError", () => {
  it("preserves a structured backend error", () => {
    const error = toBackendError({
      code: "INVALID_PARAMS",
      message: "message must be a string",
      data: { field: "message" },
    });

    expect(error).toMatchObject({
      name: "BackendError",
      code: "INVALID_PARAMS",
      message: "message must be a string",
      data: { field: "message" },
    });
  });

  it("accepts Tauri errors serialized as JSON strings", () => {
    const error = toBackendError('{"code":"REQUEST_TIMEOUT","message":"too slow"}');

    expect(error.code).toBe("REQUEST_TIMEOUT");
    expect(error.message).toBe("too slow");
  });
});
