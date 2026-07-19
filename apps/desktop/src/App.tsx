import { useState } from "react";
import { useCosync } from "./hooks/useCosync";
import { DeviceCard, ErrorBanner, PeerList, StatusBadge } from "./components/ui";
import { ClipboardPanel } from "./components/ClipboardPanel";
import { FileTransferPanel } from "./components/FileTransferPanel";

type Tab = "devices" | "clipboard" | "files";

interface TabDef {
  id: Tab;
  label: string;
  icon: React.ReactNode;
  badge?: number;
}

export default function App() {
  const {
    state,
    startDiscovery,
    stopDiscovery,
    pairDevice,
    sendClipboard,
    loadClipboardHistory,
    clearClipboardHistory,
    sendFile,
    dismissTransfer,
    dismissError,
  } = useCosync();

  const [activeTab, setActiveTab] = useState<Tab>("devices");

  const isScanning =
    state.connectionStatus === "discovering" ||
    state.connectionStatus === "connected";
  const isConnected = state.connectionStatus === "connected";

  const tabs: TabDef[] = [
    {
      id: "devices",
      label: "Devices",
      icon: (
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
          <line x1="8" y1="21" x2="16" y2="21" />
          <line x1="12" y1="17" x2="12" y2="21" />
        </svg>
      ),
      badge: state.discoveredPeers.length,
    },
    {
      id: "clipboard",
      label: "Clipboard",
      icon: (
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
          <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
        </svg>
      ),
    },
    {
      id: "files",
      label: "Files",
      icon: (
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
      ),
      badge:
        state.activeTransfers.filter((t) => t.status === "incoming").length ||
        undefined,
    },
  ];

  return (
    <div className="flex h-screen flex-col bg-gray-950 text-white">
      {/* ── Header ─────────────────────────────────────── */}
      <header className="border-b border-white/10 px-6 py-3 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-bold tracking-tight">Cosync</h1>
          <StatusBadge status={state.connectionStatus} />
        </div>
        {state.deviceInfo && <DeviceCard info={state.deviceInfo} />}
      </header>

      {/* ── Tab bar ─────────────────────────────────────── */}
      <nav className="flex gap-1 border-b border-white/10 px-6 shrink-0">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`relative flex items-center gap-2 px-4 py-3 text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? "text-white"
                : "text-gray-500 hover:text-gray-300"
            }`}
          >
            {tab.icon}
            <span>{tab.label}</span>
            {tab.badge ? (
              <span className="flex h-5 min-w-5 items-center justify-center rounded-full bg-indigo-500/20 px-1.5 text-xs text-indigo-400">
                {tab.badge}
              </span>
            ) : null}
            {activeTab === tab.id && (
              <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-indigo-500" />
            )}
          </button>
        ))}
      </nav>

      {/* ── Main content ───────────────────────────────── */}
      <main className="flex-1 overflow-y-auto px-6 py-5">
        {/* Error banner */}
        {state.errorMessage && (
          <div className="mb-4">
            <ErrorBanner
              message={state.errorMessage}
              onDismiss={dismissError}
            />
          </div>
        )}

        {/* Devices tab */}
        {activeTab === "devices" && (
          <>
            <div className="flex gap-3 mb-5">
              {!isScanning ? (
                <button
                  onClick={startDiscovery}
                  className="rounded-xl bg-indigo-600 px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-indigo-500 active:bg-indigo-700"
                >
                  Start Scanning
                </button>
              ) : (
                <button
                  onClick={stopDiscovery}
                  className="rounded-xl border border-white/20 bg-transparent px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-white/5"
                >
                  Stop Scanning
                </button>
              )}
            </div>
            <h2 className="mb-3 text-sm font-medium uppercase tracking-wider text-gray-400">
              Discovered Devices
            </h2>
            <PeerList
              peers={state.discoveredPeers}
              onPair={pairDevice}
              disabled={state.connectionStatus !== "discovering"}
            />
          </>
        )}

        {/* Clipboard tab */}
        {activeTab === "clipboard" && (
          <ClipboardPanel
            history={state.clipboardHistory}
            isConnected={isConnected}
            onSend={sendClipboard}
            onLoad={loadClipboardHistory}
            onClear={clearClipboardHistory}
          />
        )}

        {/* Files tab */}
        {activeTab === "files" && (
          <FileTransferPanel
            transfers={state.activeTransfers}
            isConnected={isConnected}
            onSend={sendFile}
            onDismiss={dismissTransfer}
          />
        )}
      </main>

      {/* ── Footer ─────────────────────────────────────── */}
      <footer className="border-t border-white/10 px-6 py-2.5 text-center text-xs text-gray-700 shrink-0">
        Cosync v0.1.0 — LAN device sync via QUIC
      </footer>
    </div>
  );
}