import { afterEach, describe, expect, it, vi } from "vitest";
import { apiFetch, HttpError } from "./fetch";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("apiFetch", () => {
  it("returns a parsed JSON response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify({ value: 42 }), {
          headers: { "content-type": "application/json" },
        }),
      ),
    );

    await expect(apiFetch<{ value: number }>("https://example.test")).resolves.toEqual({
      value: 42,
    });
  });

  it("preserves the response body on HTTP errors", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response("not found", { status: 404 })));

    const request = apiFetch("https://example.test/missing");
    await expect(request).rejects.toBeInstanceOf(HttpError);
    await expect(request).rejects.toMatchObject({ status: 404, body: "not found" });
  });
});
