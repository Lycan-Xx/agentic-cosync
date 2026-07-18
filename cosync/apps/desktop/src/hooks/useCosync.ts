import { useEffect, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  FrontendEvent,
  DiscoveredPeerView,
  DeviceInfo,
  ClipboardEntry,
  FileTransferProgress,
} from "../types/events";
import * as cmd from "../lib/commands";

// ── App state ──────────────────────────────────────────────────────

export type ConnectionStatus =
  | "idle"
  | "discovering"
  | "pairing"
  | "connected"
  | "error";

export interface AppState {
  connectionStatus: ConnectionStatus;
  deviceInfo: DeviceInfo | null;
  discoveredPeers: DiscoveredPeerView[];
  errorMessage: string | null;
  // Clipboard
  clipboardHistory: ClipboardEntry[];
  // File transfers
  activeTransfers: FileTransferProgress[];
}

// ── Hook ───────────────────────────────────────────────────────────

export function useCosync() {
  const [state, setState] = useState<AppState>({
    connectionStatus: "idle",
    deviceInfo: null,
    discoveredPeers: [],
    errorMessage: null,
    clipboardHistory: [],
    activeTransfers: [],
  });

  const setStatus = useCallback((status: ConnectionStatus) => {
    setState((prev) => ({
      ...prev,
      connectionStatus: status,
      errorMessage: null,
    }));
  }, []);

  // Load device info on mount
  useEffect(() => {
    cmd
      .getDeviceInfo()
      .then((info) => {
        setState((prev) => ({ ...prev, deviceInfo: info }));
      })
      .catch((e) => {
        console.error("Failed to load device info:", e);
      });
  }, []);

  // Listen for all cosync events
  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    (async () => {
      const unlisten = await listen<FrontendEvent>(
        "cosync://event",
        (event) => {
          const fe = event.payload;

          switch (fe.type) {
            case "ConnectionStateChanged":
              setState((prev) => ({
                ...prev,
                connectionStatus:
                  fe.data.state.toLowerCase() as ConnectionStatus,
              }));
              break;

            case "DeviceFound":
              setState((prev) => {
                if (
                  prev.discoveredPeers.some(
                    (p) => p.fingerprint === fe.data.fingerprint,
                  )
                ) {
                  return prev;
                }
                return {
                  ...prev,
                  discoveredPeers: [...prev.discoveredPeers, fe.data],
                };
              });
              break;

            case "DeviceLost":
              setState((prev) => ({
                ...prev,
                discoveredPeers: prev.discoveredPeers.filter(
                  (p) => p.device_name !== fe.data.device_name,
                ),
              }));
              break;

            case "ClipboardReceived": {
              const newEntry: ClipboardEntry = {
                id: Date.now(),
                content: fe.data.content,
                content_type: "text/plain",
                source_device_id: fe.data.source || null,
                created_at: new Date().toISOString(),
              };
              setState((prev) => ({
                ...prev,
                clipboardHistory: [newEntry, ...prev.clipboardHistory],
              }));
              break;
            }

            case "FileIncoming": {
              const transfer: FileTransferProgress = {
                transfer_id: fe.data.transfer_id,
                file_name: fe.data.file_name,
                file_size: fe.data.file_size,
                chunks_received: 0,
                total_chunks: 0,
                status: "incoming",
              };
              setState((prev) => ({
                ...prev,
                activeTransfers: [...prev.activeTransfers, transfer],
              }));
              break;
            }

            case "FileProgress": {
              setState((prev) => ({
                ...prev,
                activeTransfers: prev.activeTransfers.map((t) =>
                  t.transfer_id === fe.data.transfer_id
                    ? {
                        ...t,
                        chunks_received: fe.data.chunk_index + 1,
                        total_chunks: Math.max(
                          fe.data.total_chunks,
                          t.total_chunks,
                        ),
                      }
                    : t,
                ),
              }));
              break;
            }

            case "FileComplete": {
              setState((prev) => ({
                ...prev,
                activeTransfers: prev.activeTransfers.map((t) =>
                  t.transfer_id === fe.data.transfer_id
                    ? {
                        ...t,
                        status: fe.data.success ? "complete" : "error",
                        path: fe.data.path || undefined,
                        error: fe.data.success ? undefined : "Transfer failed",
                      }
                    : t,
                ),
              }));
              break;
            }

            case "Error":
              setState((prev) => ({
                ...prev,
                errorMessage: fe.data.message,
                connectionStatus: "error",
              }));
              break;

            default:
              console.warn("Unknown event type:", fe);
          }
        },
      );

      unlisteners.push(unlisten);
    })();

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, []);

  // ── Actions ───────────────────────────────────────────────────────

  const startDiscovery = useCallback(async () => {
    setStatus("discovering");
    try {
      await cmd.startDiscovery();
    } catch (e) {
      setState((prev) => ({
        ...prev,
        errorMessage: String(e),
        connectionStatus: "error",
      }));
    }
  }, [setStatus]);

  const stopDiscovery = useCallback(async () => {
    try {
      await cmd.stopDiscovery();
      setStatus("idle");
    } catch (e) {
      setState((prev) => ({ ...prev, errorMessage: String(e) }));
    }
  }, [setStatus]);

  const pairDevice = useCallback(
    async (peer: DiscoveredPeerView) => {
      setStatus("pairing");
      try {
        const ip = peer.addresses[0];
        if (!ip) throw new Error("No IP address for peer");
        await cmd.pairWithDevice(ip, peer.port, peer.fingerprint);
        setStatus("connected");
      } catch (e) {
        setState((prev) => ({
          ...prev,
          errorMessage: String(e),
          connectionStatus: "error",
        }));
      }
    },
    [setStatus],
  );

  const sendClipboard = useCallback(async (content: string) => {
    try {
      await cmd.sendClipboard(content);
    } catch (e) {
      setState((prev) => ({ ...prev, errorMessage: String(e) }));
    }
  }, []);

  const loadClipboardHistory = useCallback(async () => {
    try {
      const entries = await cmd.getClipboardHistory();
      setState((prev) => ({
        ...prev,
        clipboardHistory: entries,
      }));
    } catch (e) {
      console.error("Failed to load clipboard history:", e);
    }
  }, []);

  const clearClipboardHistory = useCallback(async () => {
    try {
      await cmd.clearClipboardHistory();
      setState((prev) => ({ ...prev, clipboardHistory: [] }));
    } catch (e) {
      console.error("Failed to clear clipboard history:", e);
    }
  }, []);

  const sendFile = useCallback(
    async (filePath: string) => {
      try {
        const result = await cmd.sendFile(filePath);
        // Add the outgoing transfer to the UI
        const transfer: FileTransferProgress = {
          transfer_id: result.transfer_id,
          file_name: result.file_name,
          file_size: result.file_size,
          chunks_received: 0,
          total_chunks: result.total_chunks,
          status: "complete",
        };
        setState((prev) => ({
          ...prev,
          activeTransfers: [transfer, ...prev.activeTransfers],
        }));
      } catch (e) {
        setState((prev) => ({ ...prev, errorMessage: String(e) }));
      }
    },
    [],
  );

  const dismissTransfer = useCallback((transferId: string) => {
    setState((prev) => ({
      ...prev,
      activeTransfers: prev.activeTransfers.filter(
        (t) => t.transfer_id !== transferId,
      ),
    }));
  }, []);

  return {
    state,
    startDiscovery,
    stopDiscovery,
    pairDevice,
    sendClipboard,
    loadClipboardHistory,
    clearClipboardHistory,
    sendFile,
    dismissTransfer,
    dismissError: useCallback(() => {
      setState((prev) => ({ ...prev, errorMessage: null }));
    }, []),
  };
}