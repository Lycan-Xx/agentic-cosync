import { useCosync } from "./hooks/useCosync";
import { DeviceCard, ErrorBanner, PeerList, StatusBadge } from "./components/ui";

export default function App() {
  const { state, startDiscovery, stopDiscovery, pairDevice, dismissError } = useCosync();

  const isScanning = state.connectionStatus === "discovering" || state.connectionStatus === "connected";

  return (
    <div className="min-h-screen bg-gray-950 text-white flex flex-col">
      {/* ── Header ─────────────────────────────────────── */}
      <header className="border-b border-white/10 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-bold tracking-tight">
            Cosync
          </h1>
          <StatusBadge status={state.connectionStatus} />
        </div>
        {state.deviceInfo && <DeviceCard info={state.deviceInfo} />}
      </header>

      {/* ── Main ───────────────────────────────────────── */}
      <main className="flex-1 px-6 py-6 max-w-xl mx-auto w-full">
        {/* Error banner */}
        {state.errorMessage && (
          <div className="mb-4">
            <ErrorBanner
              message={state.errorMessage}
              onDismiss={dismissError}
            />
          </div>
        )}

        {/* Action buttons */}
        <div className="flex gap-3 mb-6">
          {!isScanning ? (
            <button
              onClick={startDiscovery}
              className="flex-1 rounded-xl bg-indigo-600 px-4 py-3 text-sm font-semibold text-white transition-colors hover:bg-indigo-500 active:bg-indigo-700"
            >
              Start Scanning
            </button>
          ) : (
            <button
              onClick={stopDiscovery}
              className="flex-1 rounded-xl border border-white/20 bg-transparent px-4 py-3 text-sm font-semibold text-white transition-colors hover:bg-white/5"
            >
              Stop Scanning
            </button>
          )}
        </div>

        {/* Discovered peers */}
        <section>
          <h2 className="mb-3 text-sm font-medium uppercase tracking-wider text-gray-400">
            Discovered Devices
          </h2>
          <PeerList
            peers={state.discoveredPeers}
            onPair={pairDevice}
            disabled={state.connectionStatus !== "discovering"}
          />
        </section>
      </main>

      {/* ── Footer ─────────────────────────────────────── */}
      <footer className="border-t border-white/10 px-6 py-3 text-center text-xs text-gray-600">
        Cosync v0.1.0 — LAN device sync via QUIC
      </footer>
    </div>
  );
}