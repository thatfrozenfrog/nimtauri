import { invoke } from "@tauri-apps/api/core";
import type {
  AddResult,
  BackendErrorPayload,
  BackendStatus,
  PingResult,
  SystemInfo,
} from "./backend.types";
import { BackendError } from "./backend.types";

function isBackendErrorPayload(value: unknown): value is BackendErrorPayload {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    typeof value.code === "string" &&
    "message" in value &&
    typeof value.message === "string"
  );
}

export function toBackendError(reason: unknown): BackendError {
  if (reason instanceof BackendError) return reason;

  let payload = reason;
  if (typeof reason === "string") {
    try {
      payload = JSON.parse(reason);
    } catch {
      return new BackendError("TAURI_INVOKE_ERROR", reason);
    }
  }

  if (isBackendErrorPayload(payload)) {
    return new BackendError(payload.code, payload.message, payload.data);
  }

  if (reason instanceof Error) {
    return new BackendError("TAURI_INVOKE_ERROR", reason.message);
  }

  return new BackendError("TAURI_INVOKE_ERROR", "an unknown backend error occurred", reason);
}

async function invokeBackend<TResult>(
  command: string,
  args?: Record<string, unknown>,
): Promise<TResult> {
  try {
    return await invoke<TResult>(command, args);
  } catch (reason) {
    throw toBackendError(reason);
  }
}

export async function callBackend<TParams, TResult>(
  method: string,
  params: TParams,
): Promise<TResult> {
  return invokeBackend<TResult>("backend_call", { method, params });
}

export const ping = (message: string) =>
  callBackend<{ message: string }, PingResult>("ping", { message });

export const getSystemInfo = () =>
  callBackend<Record<string, never>, SystemInfo>("system.info", {});

export const addNumbers = (a: number, b: number) =>
  callBackend<{ a: number; b: number }, AddResult>("math.add", { a, b });

export const getBackendStatus = () => invokeBackend<BackendStatus>("backend_status");

export const restartBackend = () => invokeBackend<BackendStatus>("backend_restart");
