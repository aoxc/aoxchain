// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

export type Track = {
  name: string
  percent: number
  summary: string
  status: 'ready' | 'in-progress'
}

export type LaunchBlocker = {
  title: string
  detail: string
  command: string
}

export type FileStatus = {
  label: string
  path: string
  exists: boolean
}

export type AreaProgress = {
  name: string
  percent: number
  detail: string
  status: 'ready' | 'in-progress'
}

export type NodeControl = {
  id: string
  role: string
  status: 'online' | 'degraded' | 'offline' | 'blocked'
  chainId: string
  listenAddr: string
  rpcAddr: string
  peerCount: number
  securityMode: string
  command: string
}

export type WalletSurface = {
  title: string
  route: string
  status: 'connected' | 'attention' | 'locked'
  addressHint: string
  command: string
  detail: string
}

export type TelemetrySurface = {
  title: string
  status: 'ready' | 'blocked'
  target: string
  detail: string
}

export type ReportAsset = {
  title: string
  status: 'ready' | 'queued'
  path: string
  detail: string
}

export type CommandPreset = {
  title: string
  command: string
  intent: string
}

export type DatabaseSurface = {
  title: string
  status: string
  path: string
  detail: string
}

export type LogSurface = {
  title: string
  status: string
  path: string
  detail: string
}

export type ExplorerSurface = {
  title: string
  status: string
  target: string
  detail: string
}

export type TerminalSurface = {
  title: string
  command: string
  detail: string
}

export type ControlCenterSnapshot = {
  stage: string
  verdict: string
  overallPercent: number
  profile: string
  summary: string
  tracks: Track[]
  blockers: LaunchBlocker[]
  files: FileStatus[]
  areas: AreaProgress[]
  nodes: NodeControl[]
  wallets: WalletSurface[]
  telemetry: TelemetrySurface[]
  reports: ReportAsset[]
  commands: CommandPreset[]
  databases: DatabaseSurface[]
  logs: LogSurface[]
  explorer: ExplorerSurface[]
  terminals: TerminalSurface[]
}
