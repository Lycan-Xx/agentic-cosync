/** Shapes returned by Tauri commands. */

export interface DeviceInfo {
  device_name: string;
  fingerprint: string;
}

export interface PairedDeviceView {
  device_id: string;
  device_name: string;
  fingerprint: string;
  last_known_ip: string | null;
  last_seen_at: string;
}

export interface DiscoveredPeerView {
  device_name: string;
  fingerprint: string;
  addresses: string[];
  port: number;
}

/**
 * Tagged union emitted on `cosync://event`.
 * Mirrors the Rust `FrontendEvent` enum exactly.
 */
export type FrontendEvent =
  | { type: "ConnectionStateChanged"; data: { state: string } }
  | { type: "DeviceFound"; data: DiscoveredPeerView }
  | { type: "DeviceLost"; data: { device_name: string } }
  | { type: "PairingRequest"; data: { device_name: string; fingerprint: string } }
  | { type: "ClipboardReceived"; data: { content: string; source: string } }
  | { type: "FileIncoming"; data: { transfer_id: string; file_name: string; file_size: number } }
  | { type: "FileProgress"; data: { transfer_id: string; chunk_index: number; total_chunks: number } }
  | { type: "FileComplete"; data: { transfer_id: string; success: boolean; path: string } }
  | { type: "NotificationReceived"; data: { package_name: string; title: string; text: string } }
  | { type: "Error"; data: { message: string } };