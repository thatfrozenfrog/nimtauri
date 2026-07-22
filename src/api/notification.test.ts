import { beforeEach, describe, expect, it, vi } from "vitest";
import { sendBackendNotification } from "./notification";

const notification = vi.hoisted(() => ({
  isPermissionGranted: vi.fn(),
  requestPermission: vi.fn(),
  sendNotification: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: notification.isPermissionGranted,
  requestPermission: notification.requestPermission,
  sendNotification: notification.sendNotification,
}));

describe("sendBackendNotification", () => {
  beforeEach(() => vi.resetAllMocks());

  it("sends a notification when permission is already granted", async () => {
    notification.isPermissionGranted.mockResolvedValue(true);

    await expect(sendBackendNotification()).resolves.toBe(true);

    expect(notification.requestPermission).not.toHaveBeenCalled();
    expect(notification.sendNotification).toHaveBeenCalledWith({
      title: "Nim backend",
      body: "Your notification button is working.",
    });
  });

  it("does not send a notification when the permission request is denied", async () => {
    notification.isPermissionGranted.mockResolvedValue(false);
    notification.requestPermission.mockResolvedValue("denied");

    await expect(sendBackendNotification()).resolves.toBe(false);

    expect(notification.sendNotification).not.toHaveBeenCalled();
  });
});
