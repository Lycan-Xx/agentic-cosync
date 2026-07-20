import { invoke } from "@tauri-apps/api/core";
import type {
  DeviceInfo,
  PairedDeviceView,
  ClipboardEntry,
  SendFileResult,
} from "../types/events";

/** Rust Tauri commands — typed wrappers around `invoke`. */

export async function getDeviceInfo(): Promise<DeviceInfo> {
  return invoke<DeviceInfo>("get_device_info");
}

export async function getDeviceFingerprint(): Promise<string> {
  return invoke<string>("get_device_fingerprint");
}

export async function getConnectionState(): Promise<string> {
  return invoke<string>("get_connection_state");
}

export async function startDiscovery(): Promise<void> {
  return invoke<void>("start_discovery");
}

export async function stopDiscovery(): Promise<void> {
  return invoke<void>("stop_discovery");
}

export async function pairWithDevice(
  peerIp: string,
  peerPort: number,
  peerFingerprint: string,
): Promise<void> {
  return invoke<void>("pair_with_device", {
    peerIp,
    peerPort,
    peerFingerprint,
  });
}

export async function unpairDevice(deviceId: string): Promise<void> {
  return invoke<void>("unpair_device", { deviceId });
}

export async function getPairedDevices(): Promise<PairedDeviceView[]> {
  return invoke<PairedDeviceView[]>("get_paired_devices");
}

// ── Clipboard commands ──────────────────────────────────────────

export async function getClipboardHistory(): Promise<ClipboardEntry[]> {
  return invoke<ClipboardEntry[]>("get_clipboard_history");
}

export async function sendClipboard(content: string): Promise<void> {
  return invoke<void>("send_clipboard", { content });
}

export async function deleteClipboardEntry(id: number): Promise<void> {
  return invoke<void>("delete_clipboard_entry", { id });
}

export async function clearClipboardHistory(): Promise<void> {
  return invoke<void>("clear_clipboard_history");
}

export async function startClipboardMonitor(): Promise<void> {
  return invoke<void>("start_clipboard_monitor");
}

export async function stopClipboardMonitor(): Promise<void> {
  return invoke<void>("stop_clipboard_monitor");
}

// ── File transfer commands ───────────────────────────────────────

export async function sendFile(filePath: string): Promise<SendFileResult> {
  return invoke<SendFileResult>("send_file", { filePath });
}

export async function openFileInFolder(filePath: string): Promise<void> {
  return invoke<void>("open_file_in_folder", { filePath });
}