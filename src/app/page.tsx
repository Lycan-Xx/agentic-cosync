'use client'

import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { motion, AnimatePresence } from 'framer-motion'

function ArrowLeftIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="m12 19-7-7 7-7" /><path d="M19 12H5" />
    </svg>
  )
}

function MonitorIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" className={className}>
      <rect width="20" height="14" x="2" y="3" rx="2" /><line x1="8" x2="16" y1="21" y2="21" /><line x1="12" x2="12" y1="17" y2="21" />
    </svg>
  )
}

function SmartphoneIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" className={className}>
      <rect width="14" height="20" x="5" y="2" rx="2" ry="2" /><path d="M12 18h.01" />
    </svg>
  )
}

function DownloadIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" x2="12" y1="15" y2="3" />
    </svg>
  )
}

function GithubIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className={className}>
      <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
    </svg>
  )
}

function ClipboardIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" className={className}>
      <rect width="8" height="4" x="8" y="2" rx="1" ry="1" /><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" /><path d="M12 11h4" /><path d="M12 16h4" /><path d="M8 11h.01" /><path d="M8 16h.01" />
    </svg>
  )
}

function FileIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" /><path d="M14 2v4a2 2 0 0 0 2 2h4" />
    </svg>
  )
}

function BellIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" /><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" />
    </svg>
  )
}

function ShieldIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" /><path d="m9 12 2 2 4-4" />
    </svg>
  )
}

/* ─── Landing Page ─── */
function LandingPage({ onDownload }: { onDownload: () => void }) {
  return (
    <motion.main
      key="landing"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
      className="min-h-screen flex flex-col items-center justify-center px-6 relative overflow-hidden"
    >
      {/* Subtle radial glow behind the logo */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] bg-emerald-500/5 rounded-full blur-3xl pointer-events-none" />

      {/* Logo / Mark */}
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ delay: 0.1, duration: 0.5, ease: 'easeOut' }}
        className="mb-10"
      >
        <div className="w-20 h-20 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center">
          <svg width="44" height="44" viewBox="0 0 44 44" fill="none" className="text-emerald-400">
            <path d="M22 4L8 12v12c0 8 6 14.5 14 16 8-1.5 14-8 14-16V12L22 4z" stroke="currentColor" strokeWidth="2" strokeLinejoin="round" fill="none" />
            <path d="M15 22l4 4 10-10" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" fill="none" />
          </svg>
        </div>
      </motion.div>

      {/* Title */}
      <motion.h1
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.2, duration: 0.5 }}
        className="text-5xl sm:text-6xl font-bold tracking-tight text-center"
      >
        Cosync
      </motion.h1>

      {/* Tagline */}
      <motion.p
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.35, duration: 0.5 }}
        className="mt-4 text-lg sm:text-xl text-muted-foreground text-center max-w-md leading-relaxed"
      >
        Clipboard, files &amp; notifications — synced across your devices over LAN.
      </motion.p>

      {/* Feature pills */}
      <motion.div
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.5, duration: 0.5 }}
        className="mt-8 flex flex-wrap items-center justify-center gap-2"
      >
        <span className="inline-flex items-center gap-1.5 text-sm text-muted-foreground bg-muted/50 border border-border/50 rounded-full px-3.5 py-1.5">
          <ClipboardIcon className="size-3.5 text-emerald-400" /> Clipboard
        </span>
        <span className="inline-flex items-center gap-1.5 text-sm text-muted-foreground bg-muted/50 border border-border/50 rounded-full px-3.5 py-1.5">
          <FileIcon className="size-3.5 text-emerald-400" /> Files
        </span>
        <span className="inline-flex items-center gap-1.5 text-sm text-muted-foreground bg-muted/50 border border-border/50 rounded-full px-3.5 py-1.5">
          <BellIcon className="size-3.5 text-emerald-400" /> Notifications
        </span>
        <span className="inline-flex items-center gap-1.5 text-sm text-muted-foreground bg-muted/50 border border-border/50 rounded-full px-3.5 py-1.5">
          <ShieldIcon className="size-3.5 text-emerald-400" /> E2E Encrypted
        </span>
      </motion.div>

      {/* CTA Button */}
      <motion.div
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.65, duration: 0.5 }}
        className="mt-12"
      >
        <Button
          size="lg"
          onClick={onDownload}
          className="h-14 px-10 text-base font-semibold bg-emerald-600 hover:bg-emerald-500 text-white rounded-xl shadow-lg shadow-emerald-600/20 cursor-pointer"
        >
          <DownloadIcon className="size-5" />
          Download
        </Button>
      </motion.div>

      {/* Subtext */}
      <motion.p
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.85, duration: 0.5 }}
        className="mt-4 text-xs text-muted-foreground/60 text-center"
      >
        No cloud &middot; No accounts &middot; Open source
      </motion.p>

      {/* Footer */}
      <footer className="absolute bottom-6 text-xs text-muted-foreground/40">
        Built with Rust, QUIC &amp; React
      </footer>
    </motion.main>
  )
}

/* ─── Download Page ─── */
function DownloadPage({ onBack }: { onBack: () => void }) {
  const platforms = [
    {
      name: 'Desktop',
      description: 'Linux, macOS, Windows',
      icon: MonitorIcon,
      badge: 'Tauri v2',
      details: 'Electron alternative — native binary, ~8MB, zero bloat. Rust backend paired with a React/TypeScript frontend shell.',
      link: '#',
      available: true,
    },
    {
      name: 'Android',
      description: 'Android 8.0+ (API 26)',
      icon: SmartphoneIcon,
      badge: 'React Native',
      details: 'Expo bare workflow with UniFFI → Kotlin bridge to the same Rust core. Foreground service keeps sync alive.',
      link: '#',
      available: false,
    },
  ]

  return (
    <motion.main
      key="download"
      initial={{ opacity: 0, y: 30 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.35 }}
      className="min-h-screen flex flex-col items-center px-6 py-16"
    >
      {/* Back button */}
      <div className="w-full max-w-2xl">
        <Button
          variant="ghost"
          size="sm"
          onClick={onBack}
          className="text-muted-foreground hover:text-foreground -ml-2 mb-12 cursor-pointer"
        >
          <ArrowLeftIcon className="size-4" />
          Back
        </Button>
      </div>

      {/* Heading */}
      <div className="w-full max-w-2xl text-center mb-12">
        <h1 className="text-3xl sm:text-4xl font-bold tracking-tight">Download Cosync</h1>
        <p className="mt-3 text-muted-foreground max-w-md mx-auto leading-relaxed">
          Pick your platform. Both apps share the same Rust core — your clipboard, files, and notifications sync seamlessly between them over your local network.
        </p>
      </div>

      {/* Platform cards */}
      <div className="w-full max-w-2xl grid gap-4">
        {platforms.map((p) => (
          <motion.div
            key={p.name}
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.15, duration: 0.35 }}
            className={`group relative rounded-2xl border p-6 transition-colors ${
              p.available
                ? 'border-border/60 bg-card/50 hover:border-emerald-500/40 hover:bg-card'
                : 'border-border/30 bg-card/20 opacity-70'
            }`}
          >
            <div className="flex items-start gap-5">
              <div className={`shrink-0 w-12 h-12 rounded-xl flex items-center justify-center ${
                p.available ? 'bg-emerald-500/10 text-emerald-400' : 'bg-muted/50 text-muted-foreground'
              }`}>
                <p.icon className="size-6" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2.5 flex-wrap">
                  <h2 className="text-lg font-semibold">{p.name}</h2>
                  <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground/70 bg-muted/60 rounded-full px-2 py-0.5">
                    {p.badge}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground mt-0.5">{p.description}</p>
                <p className="text-sm text-muted-foreground/70 mt-2 leading-relaxed">{p.details}</p>
              </div>
              <div className="shrink-0 self-center">
                {p.available ? (
                  <Button
                    asChild
                    size="lg"
                    className="bg-emerald-600 hover:bg-emerald-500 text-white rounded-xl cursor-pointer"
                  >
                    <a href={p.link} download>
                      <DownloadIcon className="size-4" />
                      <span className="hidden sm:inline">Download</span>
                    </a>
                  </Button>
                ) : (
                  <Button variant="outline" size="lg" disabled className="rounded-xl">
                    Coming Soon
                  </Button>
                )}
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Info section */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.4, duration: 0.5 }}
        className="w-full max-w-2xl mt-12 rounded-2xl border border-border/40 bg-card/30 p-6"
      >
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground mb-4">Before you start</h3>
        <ul className="space-y-3 text-sm text-muted-foreground leading-relaxed">
          <li className="flex items-start gap-2.5">
            <span className="text-emerald-400 mt-0.5 shrink-0">1.</span>
            <span>Both devices must be on the <strong className="text-foreground font-medium">same Wi-Fi network</strong>. Cosync works over LAN only — no internet required.</span>
          </li>
          <li className="flex items-start gap-2.5">
            <span className="text-emerald-400 mt-0.5 shrink-0">2.</span>
            <span>Install on both devices, then <strong className="text-foreground font-medium">scan the QR code</strong> on one device from the other to pair them via a pinned self-signed certificate exchange.</span>
          </li>
          <li className="flex items-start gap-2.5">
            <span className="text-emerald-400 mt-0.5 shrink-0">3.</span>
            <span>All traffic is encrypted with <strong className="text-foreground font-medium">TLS 1.3 over QUIC</strong>. Your data never leaves your local network.</span>
          </li>
        </ul>
      </motion.div>

      {/* Source link */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.55, duration: 0.5 }}
        className="mt-8"
      >
        <a
          href="https://github.com"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          <GithubIcon className="size-4" />
          View source on GitHub
        </a>
      </motion.div>

      {/* Footer */}
      <footer className="mt-auto pt-16 text-xs text-muted-foreground/40 text-center">
        Cosync v0.1.0 &middot; MIT License
      </footer>
    </motion.main>
  )
}

/* ─── App Root ─── */
export default function Home() {
  const [view, setView] = useState<'landing' | 'download'>('landing')

  return (
    <AnimatePresence mode="wait">
      {view === 'landing' ? (
        <LandingPage onDownload={() => setView('download')} />
      ) : (
        <DownloadPage onBack={() => setView('landing')} />
      )}
    </AnimatePresence>
  )
}