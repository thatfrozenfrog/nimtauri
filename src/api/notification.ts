import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

const title = "Nim backend";
const body = "Your notification button is working.";

/** Ask for OS permission when needed, then display the backend-demo notification. */
export async function sendBackendNotification(): Promise<boolean> {
  let allowed = await isPermissionGranted();
  if (!allowed) {
    allowed = (await requestPermission()) === "granted";
  }

  if (allowed) sendNotification({ title, body });
  return allowed;
}
