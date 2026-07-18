import { invoke } from "@tauri-apps/api/core";
import type { DeviceInfo, PairedDeviceView } from "../types/events";

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

export async function getClipboardHistory(): Promise<string[]> {
  return invoke<string[]>("get_clipboard_history");
}