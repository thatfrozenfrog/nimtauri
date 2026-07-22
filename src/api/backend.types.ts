export type BackendState = "starting" | "ready" | "stopped" | "failed";

export interface BackendStatus {
  state: BackendState;
  version?: string;
  protocol?: number;
  lastError?: string;
}

export interface BackendErrorPayload {
  code: string;
  message: string;
  data?: unknown;
}

export class BackendError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly data?: unknown,
  ) {
    super(message);
    this.name = "BackendError";
  }
}

export interface PingResult {
  message: string;
  timestamp: number;
}

export interface SystemInfo {
  backend: "nim";
  backendVersion: string;
  nimVersion: string;
  protocolVersion: number;
  operatingSystem: string;
  architecture: string;
}

export interface AddResult {
  value: number;
}
