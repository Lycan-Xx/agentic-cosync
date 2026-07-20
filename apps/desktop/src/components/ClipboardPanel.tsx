import { useState, useEffect } from "react";
import type { ClipboardEntry } from "../types/events";

interface ClipboardPanelProps {
  history: ClipboardEntry[];
  isConnected: boolean;
  isMonitoring: boolean;
  onSend: (content: string) => void;
  onLoad: () => void;
  onClear: () => void;
  onDelete: (id: number) => void;
}

export function ClipboardPanel({
  history,
  isConnected,
  isMonitoring,
  onSend,
  onLoad,
  onClear,
  onDelete,
}: ClipboardPanelProps) {
  const [input, setInput] = useState("");
  const [copiedId, setCopiedId] = useState<number | null>(null);

  // Load history when the component first mounts (shows persisted entries)
  // and reload when the user connects.
  useEffect(() => {
    onLoad();
  }, [onLoad]);

  const handleSend = () => {
    const trimmed = input.trim();
    if (!trimmed) return;
    onSend(trimmed);
    setInput("");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const copyToClipboard = async (content: string, id: number) => {
    try {
      await navigator.clipboard.writeText(content);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 1500);
    } catch {
      // Clipboard API may not be available in all Tauri contexts
    }
  };

  const formatTime = (iso: string) => {
    try {
      const d = new Date(iso);
      return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    } catch {
      return "";
    }
  };

  const formatSize = (text: string) => {
    const bytes = new TextEncoder().encode(text).length;
    if (bytes < 1024) return `${bytes} B`;
    return `${(bytes / 1024).toFixed(1)} KB`;
  };

  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-medium uppercase tracking-wider text-gray-400">
            Clipboard Sync
          </h2>
          {isMonitoring && (
            <span className="inline-flex items-center gap-1 rounded-full bg-green-500/10 px-2 py-0.5 text-[10px] font-medium text-green-400">
              <span className="h-1.5 w-1.5 rounded-full bg-green-400 animate-pulse" />
              Monitoring
            </span>
          )}
        </div>
        {history.length > 0 && (
          <button
            onClick={onClear}
            className="text-xs text-gray-500 hover:text-gray-300 transition-colors"
          >
            Clear history
          </button>
        )}
      </div>

      {/* Send bar */}
      <div className="flex gap-2">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={
            isConnected
              ? "Type text and press Enter to send to all paired devices..."
              : isMonitoring
                ? "Monitoring your clipboard — copies appear below automatically..."
                : "Start scanning to enable clipboard sync"
          }
          disabled={!isConnected}
          rows={2}
          className="flex-1 resize-none rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white placeholder-gray-600 transition-colors focus:border-indigo-500/50 focus:outline-none focus:ring-1 focus:ring-indigo-500/50 disabled:opacity-40"
        />
        <button
          onClick={handleSend}
          disabled={!isConnected || !input.trim()}
          className="self-end rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-500 disabled:opacity-40 disabled:cursor-not-allowed"
        >
          Send
        </button>
      </div>

      {/* History list */}
      <div className="flex flex-col gap-1.5 max-h-96 overflow-y-auto">
        {history.length === 0 ? (
          <p className="text-center text-sm text-gray-600 py-8">
            {isMonitoring
              ? "Copy text anywhere on your computer — it will appear here."
              : isConnected
                ? "Clipboard history will appear here as items are synced."
                : "No clipboard history yet."}
          </p>
        ) : (
          history.map((entry) => (
            <div
              key={`${entry.id}-${entry.created_at}`}
              className="group flex items-start gap-3 rounded-lg border border-white/5 bg-white/[0.02] px-3 py-2.5 transition-colors hover:bg-white/5"
            >
              {/* Icon */}
              <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-indigo-500/10 text-indigo-400">
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
                  <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
                </svg>
              </div>

              {/* Content */}
              <div className="min-w-0 flex-1">
                <p className="text-sm text-gray-200 break-words whitespace-pre-wrap line-clamp-3">
                  {entry.content}
                </p>
                <div className="mt-1 flex items-center gap-2 text-xs text-gray-600">
                  <span>{formatTime(entry.created_at)}</span>
                  <span>·</span>
                  <span>{formatSize(entry.content)}</span>
                  {entry.source_device_id ? (
                    <>
                      <span>·</span>
                      <span className="text-indigo-400/70">
                        from {entry.source_device_id.slice(0, 8)}…
                      </span>
                    </>
                  ) : (
                    <>
                      <span>·</span>
                      <span className="text-gray-500">local</span>
                    </>
                  )}
                </div>
              </div>

              {/* Action buttons */}
              <div className="flex items-center gap-0.5 shrink-0 opacity-0 transition-all group-hover:opacity-100">
                {/* Copy button */}
                <button
                  onClick={() => copyToClipboard(entry.content, entry.id)}
                  className="rounded-md p-1.5 text-gray-500 hover:bg-white/10 hover:text-gray-300"
                  title="Copy to clipboard"
                >
                  {copiedId === entry.id ? (
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-green-400">
                      <polyline points="20 6 9 17 4 12" />
                    </svg>
                  ) : (
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                    </svg>
                  )}
                </button>

                {/* Delete button */}
                <button
                  onClick={() => onDelete(entry.id)}
                  className="rounded-md p-1.5 text-gray-500 hover:bg-red-500/10 hover:text-red-400"
                  title="Delete"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M3 6h18" />
                    <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                    <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                  </svg>
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </section>
  );
}