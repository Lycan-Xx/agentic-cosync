import { useState, useRef } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { FileTransferProgress } from "../types/events";

interface FileTransferPanelProps {
  transfers: FileTransferProgress[];
  isConnected: boolean;
  onSend: (filePath: string) => void;
  onDismiss: (transferId: string) => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function TransferRow({
  transfer,
  onDismiss,
}: {
  transfer: FileTransferProgress;
  onDismiss: (id: string) => void;
}) {
  const progress =
    transfer.total_chunks > 0
      ? (transfer.chunks_received / transfer.total_chunks) * 100
      : 0;

  const statusColor =
    transfer.status === "complete"
      ? "text-green-400"
      : transfer.status === "error"
        ? "text-red-400"
        : "text-yellow-400";

  const statusLabel =
    transfer.status === "incoming"
      ? "Receiving..."
      : transfer.status === "complete"
        ? "Complete"
        : "Failed";

  return (
    <div className="flex items-center gap-3 rounded-lg border border-white/5 bg-white/[0.02] px-3 py-2.5 transition-colors hover:bg-white/5">
      {/* File icon */}
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-violet-500/10 text-violet-400">
        {transfer.status === "complete" ? (
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <polyline points="16 13 12 17 8 13" />
            <line x1="12" y1="17" x2="12" y2="9" />
          </svg>
        ) : (
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="17 8 12 3 7 8" />
            <line x1="12" y1="3" x2="12" y2="15" />
          </svg>
        )}
      </div>

      {/* Info + progress */}
      <div className="min-w-0 flex-1">
        <div className="flex items-center justify-between gap-2">
          <p className="truncate text-sm font-medium text-gray-200">
            {transfer.file_name}
          </p>
          <span className={`shrink-0 text-xs font-medium ${statusColor}`}>
            {statusLabel}
          </span>
        </div>
        <div className="mt-1 flex items-center gap-2 text-xs text-gray-500">
          <span>{formatBytes(transfer.file_size)}</span>
          {transfer.status === "incoming" && transfer.total_chunks > 0 && (
            <>
              <span>·</span>
              <span>{Math.round(progress)}%</span>
            </>
          )}
        </div>

        {/* Progress bar */}
        {transfer.status === "incoming" && transfer.total_chunks > 0 && (
          <div className="mt-1.5 h-1 w-full overflow-hidden rounded-full bg-white/10">
            <div
              className="h-full rounded-full bg-gradient-to-r from-indigo-500 to-violet-500 transition-all duration-200"
              style={{ width: `${progress}%` }}
            />
          </div>
        )}
      </div>

      {/* Dismiss */}
      {(transfer.status === "complete" || transfer.status === "error") && (
        <button
          onClick={() => onDismiss(transfer.transfer_id)}
          className="shrink-0 rounded-md p-1 text-gray-500 transition-colors hover:bg-white/10 hover:text-gray-300"
          title="Dismiss"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      )}
    </div>
  );
}

export function FileTransferPanel({
  transfers,
  isConnected,
  onSend,
  onDismiss,
}: FileTransferPanelProps) {
  const [isDragOver, setIsDragOver] = useState(false);
  const [sending, setSending] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        title: "Select file to send",
      });
      if (selected && !Array.isArray(selected)) {
        setSending(true);
        await onSend(selected);
        setSending(false);
      }
    } catch {
      setSending(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      // In Tauri, drag-and-drop gives web paths, we need to convert.
      // For now, use the dialog approach.
      setSending(true);
      await onSend(files[0].name);
      setSending(false);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = () => {
    setIsDragOver(false);
  };

  const activeTransfers = transfers.filter(
    (t) => t.status === "incoming",
  );
  const completedTransfers = transfers.filter(
    (t) => t.status === "complete" || t.status === "error",
  );

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-sm font-medium uppercase tracking-wider text-gray-400">
        File Transfer
      </h2>

      {/* Send area */}
      <div
        onClick={isConnected && !sending ? handleFileSelect : undefined}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        className={`relative flex cursor-pointer flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed px-6 py-8 transition-all ${
          !isConnected
            ? "cursor-not-allowed border-white/5 bg-white/[0.01]"
            : isDragOver
              ? "border-indigo-400 bg-indigo-500/10"
              : sending
                ? "border-yellow-500/30 bg-yellow-500/5"
                : "border-white/10 bg-white/[0.02] hover:border-white/20 hover:bg-white/5"
        }`}
      >
        {sending ? (
          <>
            <div className="h-8 w-8 animate-spin rounded-full border-2 border-yellow-500 border-t-transparent" />
            <p className="text-sm text-yellow-400">Sending file...</p>
          </>
        ) : (
          <>
            <svg
              width="28"
              height="28"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              className={isConnected ? "text-gray-400" : "text-gray-700"}
            >
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            <div className="text-center">
              <p className={`text-sm ${isConnected ? "text-gray-300" : "text-gray-600"}`}>
                {isConnected
                  ? "Click to select a file or drag and drop"
                  : "Connect to a device to send files"}
              </p>
              <p className="mt-0.5 text-xs text-gray-600">
                Files are sent to all paired devices
              </p>
            </div>
          </>
        )}

        <input
          ref={fileInputRef}
          type="file"
          className="hidden"
          onChange={handleFileSelect}
        />
      </div>

      {/* Active transfers */}
      {activeTransfers.length > 0 && (
        <div className="flex flex-col gap-1.5">
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Active Transfers
          </p>
          {activeTransfers.map((t) => (
            <TransferRow key={t.transfer_id} transfer={t} onDismiss={onDismiss} />
          ))}
        </div>
      )}

      {/* Completed transfers */}
      {completedTransfers.length > 0 && (
        <div className="flex flex-col gap-1.5">
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Completed
          </p>
          {completedTransfers.map((t) => (
            <TransferRow key={t.transfer_id} transfer={t} onDismiss={onDismiss} />
          ))}
        </div>
      )}

      {transfers.length === 0 && (
        <p className="text-center text-sm text-gray-600 py-2">
          No file transfers yet.
        </p>
      )}
    </section>
  );
}