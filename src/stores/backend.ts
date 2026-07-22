import { defineStore } from "pinia";
import { getBackendStatus, restartBackend } from "@/api/backend";
import type { BackendStatus } from "@/api/backend.types";

export const useBackendStore = defineStore("backend", {
  state: (): { status: BackendStatus } => ({
    status: { state: "starting" },
  }),
  actions: {
    async initialize() {
      await this.refreshStatus();
    },
    async refreshStatus() {
      try {
        this.status = await getBackendStatus();
      } catch (error) {
        this.status = {
          state: "failed",
          lastError: error instanceof Error ? error.message : String(error),
        };
      }
    },
    async restart() {
      this.status = { state: "starting" };
      try {
        this.status = await restartBackend();
      } catch (error) {
        this.status = {
          state: "failed",
          lastError: error instanceof Error ? error.message : String(error),
        };
      }
    },
  },
});
