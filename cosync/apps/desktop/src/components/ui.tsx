import type { DiscoveredPeerView, DeviceInfo } from "../types/events";
import type { ConnectionStatus } from "../hooks/useCosync";

// ── Status badge ──────────────────────────────────────────────────

const STATUS_COLORS: Record<ConnectionStatus, string> = {
  idle: "bg-gray-500",
  discovering: "bg-yellow-500 animate-pulse",
  pairing: "bg-blue-500 animate-pulse",
  connected: "bg-green-500",
  error: "bg-red-500",
};

export function StatusBadge({ status }: { status: ConnectionStatus }) {
  return (
    <span className="inline-flex items-center gap-1.5 text-xs font-medium text-gray-300">
      <span className={`h-2 w-2 rounded-full ${STATUS_COLORS[status]}`} />
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
}

// ── Device info card ──────────────────────────────────────────────

export function DeviceCard({ info }: { info: DeviceInfo }) {
  const shortFp = info.fingerprint.slice(0, 12) + "…";

  return (
    <div className="rounded-xl border border-white/10 bg-white/5 p-4 backdrop-blur-sm">
      <p className="text-xs uppercase tracking-wider text-gray-400">This device</p>
      <p className="mt-1 text-lg font-semibold text-white">{info.device_name}</p>
      <p className="mt-1 font-mono text-xs text-gray-500">{shortFp}</p>
    </div>
  );
}

// ── Peer list ─────────────────────────────────────────────────────

export function PeerList({
  peers,
  onPair,
  disabled,
}: {
  peers: DiscoveredPeerView[];
  onPair: (peer: DiscoveredPeerView) => void;
  disabled?: boolean;
}) {
  if (peers.length === 0) {
    return (
      <p className="text-center text-sm text-gray-500 py-8">
        No devices found on your network yet.
      </p>
    );
  }

  return (
    <div className="flex flex-col gap-2">
      {peers.map((peer) => (
        <div
          key={peer.fingerprint}
          className="flex items-center justify-between rounded-lg border border-white/10 bg-white/5 px-4 py-3 transition-colors hover:bg-white/10"
        >
          <div>
            <p className="text-sm font-medium text-white">{peer.device_name}</p>
            <p className="font-mono text-xs text-gray-500">{peer.addresses[0]}:{peer.port}</p>
          </div>
          <button
            onClick={() => onPair(peer)}
            disabled={disabled}
            className="rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-indigo-500 disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Pair
          </button>
        </div>
      ))}
    </div>
  );
}

// ── Error banner ──────────────────────────────────────────────────

export function ErrorBanner({ message, onDismiss }: { message: string; onDismiss: () => void }) {
  return (
    <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
      <div className="flex items-start justify-between gap-2">
        <p>{message}</p>
        <button
          onClick={onDismiss}
          className="shrink-0 text-red-400 hover:text-red-200 transition-colors"
        >
          Dismiss
        </button>
      </div>
    </div>
  );
}