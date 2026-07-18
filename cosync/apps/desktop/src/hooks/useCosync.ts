import { useEffect, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { FrontendEvent, DiscoveredPeerView, DeviceInfo } from "../types/events";
import * as cmd from "../lib/commands";

// ── App state ──────────────────────────────────────────────────────

export type ConnectionStatus = "idle" | "discovering" | "pairing" | "connected" | "error";

export interface AppState {
  connectionStatus: ConnectionStatus;
  deviceInfo: DeviceInfo | null;
  discoveredPeers: DiscoveredPeerView[];
  errorMessage: string | null;
}

// ── Hook ───────────────────────────────────────────────────────────

export function useCosync() {
  const [state, setState] = useState<AppState>({
    connectionStatus: "idle",
    deviceInfo: null,
    discoveredPeers: [],
    errorMessage: null,
  });

  const setStatus = useCallback((status: ConnectionStatus) => {
    setState((prev) => ({ ...prev, connectionStatus: status, errorMessage: null }));
  }, []);

  // Load device info on mount
  useEffect(() => {
    cmd.getDeviceInfo().then((info) => {
      setState((prev) => ({ ...prev, deviceInfo: info }));
    }).catch((e) => {
      console.error("Failed to load device info:", e);
    });
  }, []);

  // Listen for all cosync events
  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    (async () => {
      const unlisten = await listen<FrontendEvent>("cosync://event", (event) => {
        const fe = event.payload;

        switch (fe.type) {
          case "ConnectionStateChanged":
            setState((prev) => ({
              ...prev,
              connectionStatus: fe.data.state.toLowerCase() as ConnectionStatus,
            }));
            break;

          case "DeviceFound":
            setState((prev) => {
              // Deduplicate by fingerprint
              if (prev.discoveredPeers.some((p) => p.fingerprint === fe.data.fingerprint)) {
                return prev;
              }
              return { ...prev, discoveredPeers: [...prev.discoveredPeers, fe.data] };
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

          case "Error":
            setState((prev) => ({
              ...prev,
              errorMessage: fe.data.message,
              connectionStatus: "error",
            }));
            break;

          case "ClipboardReceived":
            // Future: update clipboard UI
            console.log("Clipboard received from", fe.data.source);
            break;

          case "FileIncoming":
            console.log("File incoming:", fe.data.file_name);
            break;

          case "FileProgress":
            // Future: update progress bar
            break;

          case "FileComplete":
            console.log("File complete:", fe.data.success ? "OK" : "FAILED");
            break;

          case "NotificationReceived":
            console.log("Notification:", fe.data.title, fe.data.text);
            break;

          default:
            console.warn("Unknown event type:", fe);
        }
      });

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
      setState((prev) => ({ ...prev, errorMessage: String(e), connectionStatus: "error" }));
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
        setState((prev) => ({ ...prev, errorMessage: String(e), connectionStatus: "error" }));
      }
    },
    [setStatus],
  );

  const dismissError = useCallback(() => {
    setState((prev) => ({ ...prev, errorMessage: null }));
  }, []);

  return { state, startDiscovery, stopDiscovery, pairDevice, dismissError };
}