import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getBackendStatus, restartBackend } from "@/api/backend";
import { useBackendStore } from "./backend";

vi.mock("@/api/backend", () => ({
  getBackendStatus: vi.fn(),
  restartBackend: vi.fn(),
}));

const getStatusMock = vi.mocked(getBackendStatus);
const restartMock = vi.mocked(restartBackend);

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
});

describe("backend store", () => {
  it("loads the ready backend status", async () => {
    getStatusMock.mockResolvedValue({ state: "ready", version: "0.1.0", protocol: 1 });
    const store = useBackendStore();

    await store.initialize();

    expect(store.status).toEqual({ state: "ready", version: "0.1.0", protocol: 1 });
  });

  it("records restart failures", async () => {
    restartMock.mockRejectedValue(new Error("restart failed"));
    const store = useBackendStore();

    await store.restart();

    expect(store.status).toEqual({ state: "failed", lastError: "restart failed" });
  });
});
