// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

type Status =
  | 'ready'
  | 'in-progress'
  | 'blocked'
  | 'online'
  | 'degraded'
  | 'offline'
  | 'connected'
  | 'attention'
  | 'locked'
  | 'collecting'
  | 'queued'

type SectionKey =
  | 'overview'
  | 'mission-control'
  | 'security'
  | 'network-modes'
  | 'nodes'
  | 'wallets'
  | 'runtime'
  | 'contracts'
  | 'explorer'
  | 'telemetry'
  | 'integrations'
  | 'reports'
  | 'evidence'

type LaunchSnapshot = {
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
  workspaces?: WorkspaceSurface[]
  aiSurfaces?: AiSurface[]
  runtimeSurfaces?: RuntimeSurface[]
  contracts?: ContractSurface[]
  explorer?: ExplorerSurface[]
}

type Track = {
  name: string
  percent: number
  summary: string
  status: Extract<Status, 'ready' | 'in-progress'>
}

type LaunchBlocker = {
  title: string
  detail: string
  command: string
}

type FileStatus = {
  label: string
  path: string
  exists: boolean
}

type AreaProgress = {
  name: string
  percent: number
  detail: string
  status: Extract<Status, 'ready' | 'in-progress'>
}

type NodeControl = {
  id: string
  role: string
  status: Extract<Status, 'online' | 'degraded' | 'offline'>
  chainId: string
  listenAddr: string
  rpcAddr: string
  peerCount: number
  securityMode: string
  command: string
}

type WalletSurface = {
  title: string
  route: string
  status: Extract<Status, 'connected' | 'attention' | 'locked'>
  addressHint: string
  command: string
  detail: string
}

type TelemetrySurface = {
  title: string
  status: Extract<Status, 'ready' | 'in-progress' | 'blocked'>
  target: string
  detail: string
}

type ReportAsset = {
  title: string
  status: Extract<Status, 'ready' | 'queued'>
  path: string
  detail: string
}

type CommandPreset = {
  title: string
  command: string
  intent: string
}

type WorkspaceSurface = {
  name: string
  path: string
  category: string
  status: string
  summary: string
}

type AiSurface = {
  name: string
  area: string
  status: string
  summary: string
  command: string
}

type MissionTile = {
  title: string
  value: string
  detail: string
  status: Extract<Status, 'ready' | 'in-progress' | 'blocked'>
}

type SecurityGuardrail = {
  title: string
  value: string
  detail: string
  status: Extract<Status, 'ready' | 'in-progress' | 'blocked'>
}

type IntegrationLane = {
  title: string
  owner: string
  endpoint: string
  detail: string
  status: Extract<Status, 'ready' | 'in-progress' | 'blocked'>
}

type StreamEvent = {
  time: string
  title: string
  detail: string
  severity: Extract<Status, 'ready' | 'in-progress' | 'blocked'>
}

type QueuedAction = {
  id: string
  title: string
  command: string
  source: string
  queuedAt: string
}

type TransferScenario = {
  title: string
  from: string
  to: string
  amount: string
  feePolicy: string
  risk: 'ready' | 'in-progress' | 'blocked'
  command: string
}

type StakeScenario = {
  title: string
  validator: string
  strategy: string
  lock: string
  estimate: string
  status: 'ready' | 'in-progress' | 'blocked'
  command: string
}

type ExplorerSignal = {
  title: string
  value: string
  detail: string
  status: 'ready' | 'in-progress' | 'blocked'
}

type RuntimeSurface = {
  title: string
  status: Extract<Status, 'ready' | 'in-progress' | 'blocked'>
  ramMb: number
  target: string
  detail: string
  command: string
}

type ContractSurface = {
  name: string
  lane: string
  status: Extract<Status, 'ready' | 'in-progress' | 'blocked'>
  address: string
  vm: string
  detail: string
  command: string
}

type ExplorerSurface = {
  name: string
  status: Extract<Status, 'ready' | 'in-progress' | 'blocked'>
  endpoint: string
  detail: string
  command: string
}

type NetworkMode = {
  lane: 'devnet' | 'testnet' | 'mainnet'
  chainId: string
  rpc: string
  p2p: string
  bootstrap: string
  status: 'ready' | 'in-progress' | 'blocked'
}

type CliLane = {
  title: string
  focus: string
  commands: CommandPreset[]
}

const fallbackSnapshot: LaunchSnapshot = {
  stage: 'Desktop operations baseline',
  verdict: 'Needs operator closure',
  overallPercent: 68,
  profile: 'aoxhub.desktop.admin',
  summary:
    'AOXHub masaüstü omurgası; node, wallet, telemetry, audit ve release kanıtlarını tek kontrol merkezinde toplar.',
  tracks: [
    {
      name: 'Mainnet readiness',
      percent: 60,
      summary: 'Production kontrolleri ve release kanıtları kapanmalı.',
      status: 'in-progress',
    },
    {
      name: 'Testnet readiness',
      percent: 77,
      summary: 'AOXHub ile çekirdek node davranışı aynı kalmalı.',
      status: 'in-progress',
    },
    {
      name: 'Desktop control center',
      percent: 82,
      summary: 'Masaüstü cockpit operasyon katmanlarını tek ekranda birleştiriyor.',
      status: 'ready',
    },
  ],
  blockers: [
    {
      title: 'Three-node orchestrator wiring',
      detail: 'Node komutları güvenli adapter katmanı üzerinden finalize edilmeli.',
      command: 'configs/deterministic-testnet/launch-testnet.sh',
    },
    {
      title: 'Wallet signing guardrails',
      detail: 'Treasury ve recovery lane için imza politikaları UI içinde görünür olmalı.',
      command: 'cargo run -q -p aoxcmd -- production-audit --format json',
    },
  ],
  files: [
    { label: 'Progress report', path: 'AOXC_PROGRESS_REPORT.md', exists: true },
    { label: 'Mainnet profile', path: 'configs/mainnet.toml', exists: true },
    { label: 'Testnet profile', path: 'configs/testnet.toml', exists: true },
  ],
  areas: [
    { name: 'Network', percent: 100, detail: 'Core bağlantı yüzeyleri hazır.', status: 'ready' },
    { name: 'Identity', percent: 50, detail: 'Anahtar yaşam döngüsü kapanış bekliyor.', status: 'in-progress' },
    { name: 'Operations', percent: 72, detail: 'Audit ve closure exportları ilerliyor.', status: 'in-progress' },
  ],
  nodes: [
    {
      id: 'atlas',
      role: 'validator leader',
      status: 'online',
      chainId: 'aoxc-mainnet-candidate',
      listenAddr: '127.0.0.1:39001',
      rpcAddr: '127.0.0.1:8545',
      peerCount: 2,
      securityMode: 'release_guarded',
      command:
        'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/atlas --rounds 12 --sleep-ms 200',
    },
    {
      id: 'boreal',
      role: 'validator follower',
      status: 'online',
      chainId: 'aoxc-mainnet-candidate',
      listenAddr: '127.0.0.1:39002',
      rpcAddr: '127.0.0.1:8546',
      peerCount: 2,
      securityMode: 'release_guarded',
      command:
        'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/boreal --rounds 12 --sleep-ms 200',
    },
    {
      id: 'cypher',
      role: 'observer / telemetry anchor',
      status: 'degraded',
      chainId: 'aoxc-mainnet-candidate',
      listenAddr: '127.0.0.1:39003',
      rpcAddr: '127.0.0.1:9545',
      peerCount: 2,
      securityMode: 'test_fixture',
      command:
        'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/cypher --rounds 12 --sleep-ms 200',
    },
  ],
  wallets: [
    {
      title: 'Operator wallet',
      route: 'mainnet guarded',
      status: 'connected',
      addressHint: 'AOXC1-VAL-OPER-PRIMARY',
      command: 'aoxc key-bootstrap --profile mainnet --password <value>',
      detail: 'Validator lifecycle, governance ve acil durum aksiyonları için ana lane.',
    },
    {
      title: 'Treasury wallet',
      route: 'dual-route mainnet/testnet',
      status: 'attention',
      addressHint: 'AOXC1-TREASURY-DESKTOP',
      command: 'aoxc wallet inspect --profile mainnet',
      detail: 'Transfer öncesi policy, approval ve export görünürlüğü ister.',
    },
    {
      title: 'Recovery wallet',
      route: 'offline recovery lane',
      status: 'locked',
      addressHint: 'AOXC1-RECOVERY-ESCROW',
      command: 'aoxc diagnostics-bundle --redact',
      detail: 'Disaster recovery, key rotation ve cold-path verification lane.',
    },
  ],
  telemetry: [
    {
      title: 'Mainnet RPC',
      status: 'ready',
      target: '127.0.0.1:8545',
      detail: 'Security mode: release_guarded',
    },
    {
      title: 'Telemetry snapshot',
      status: 'blocked',
      target: 'artifacts/network-production-closure/telemetry-snapshot.json',
      detail: 'Prometheus ve alarm çıktıları closure dizinine export edilmeli.',
    },
  ],
  reports: [
    {
      title: 'Progress report',
      status: 'ready',
      path: 'AOXC_PROGRESS_REPORT.md',
      detail: 'Masaüstü kokpitin beslendiği ana readiness özeti.',
    },
    {
      title: 'Network production closure',
      status: 'queued',
      path: 'artifacts/network-production-closure',
      detail: 'Soak, telemetry ve recovery kanıtları tek yerde tutulur.',
    },
  ],
  commands: [
    {
      title: 'Bring up deterministic 3-node cluster',
      command: 'configs/deterministic-testnet/launch-testnet.sh',
      intent: 'Üç lokal node kurup masaüstü orkestrasyonunu doğrular.',
    },
    {
      title: 'Generate production audit',
      command: 'cargo run -q -p aoxcmd -- production-audit --format json',
      intent: 'Release veya transfer öncesi audit yüzeyini yeniler.',
    },
  ],
  runtimeSurfaces: [
    {
      title: 'Node runtime memory',
      status: 'in-progress',
      ramMb: 742,
      target: 'cluster/validator-lane',
      detail: 'RAM baskısı ve GC davranışı gerçek zincir yükü altında izleniyor.',
      command: 'cargo run -q -p aoxcmd -- runtime-status --trace verbose',
    },
    {
      title: 'VM execution memory',
      status: 'ready',
      ramMb: 512,
      target: 'aoxcvm/lane-router',
      detail: 'VM lane memory profile güvenlik sınırları içinde.',
      command: 'cargo test -p aoxcvm lane_memory_profile -- --nocapture',
    },
  ],
  contracts: [
    {
      name: 'System governance core',
      lane: 'system',
      status: 'ready',
      address: 'AOXC-SYS-GOV-0001',
      vm: 'native/system',
      detail: 'Ağ yönetişim kontratları production policy ile uyumlu.',
      command: 'cargo run -q -p aoxcmd -- contract-verify --profile mainnet',
    },
    {
      name: 'Treasury execution lane',
      lane: 'evm',
      status: 'in-progress',
      address: '0xA0XC...TREA',
      vm: 'evm',
      detail: 'Treasury lane allowance ve signer politikası final review bekliyor.',
      command: 'cargo run -q -p aoxcmd -- wallet inspect --profile mainnet',
    },
    {
      name: 'Recovery policy guard',
      lane: 'wasm',
      status: 'ready',
      address: 'AOXC-WASM-REC-01',
      vm: 'wasm',
      detail: 'Recovery lane cold-path kontrat akışı doğrulandı.',
      command: 'cargo test -p aoxcvm wasm_recovery_lane -- --nocapture',
    },
  ],
  explorer: [
    {
      name: 'Chain state explorer',
      status: 'ready',
      endpoint: 'http://127.0.0.1:8545',
      detail: 'Height/hash/receipt yüzeylerini gerçek zamanlı izler.',
      command: 'cargo run -q -p aoxcmd -- db-status --backend sqlite',
    },
    {
      name: 'Contract lane explorer',
      status: 'in-progress',
      endpoint: 'aoxcvm://lanes',
      detail: 'EVM/WASM/system lane hareketleri tek panelde birleştiriliyor.',
      command: 'cargo run -q -p aoxcmd -- compat-matrix --format json',
    },
    {
      name: 'Wallet event explorer',
      status: 'ready',
      endpoint: 'wallet://authority-center',
      detail: 'Operator/treasury/recovery wallet aksiyonları izlenir.',
      command: 'cargo run -q -p aoxcmd -- production-audit --format json',
    },
  ],
  workspaces: [],
  aiSurfaces: [],
}

const navigation: { key: SectionKey; label: string }[] = [
  { key: 'overview', label: 'Overview' },
  { key: 'mission-control', label: 'Mission control' },
  { key: 'security', label: 'Security' },
  { key: 'network-modes', label: 'Dev/Test/Mainnet' },
  { key: 'nodes', label: 'Nodes' },
  { key: 'wallets', label: 'Wallets' },
  { key: 'runtime', label: 'Runtime' },
  { key: 'contracts', label: 'Contracts' },
  { key: 'explorer', label: 'Explorer' },
  { key: 'telemetry', label: 'Telemetry' },
  { key: 'integrations', label: 'Integrations' },
  { key: 'reports', label: 'Reports' },
  { key: 'evidence', label: 'Evidence' },
]

function statusLabel(status: Status) {
  switch (status) {
    case 'ready':
    case 'online':
    case 'connected':
      return 'Ready'
    case 'collecting':
    case 'queued':
    case 'attention':
    case 'degraded':
    case 'in-progress':
      return 'In progress'
    case 'locked':
    case 'offline':
    case 'blocked':
      return 'Blocked'
    default:
      return 'In progress'
  }
}

function statusTone(status: Status) {
  switch (status) {
    case 'online':
    case 'connected':
      return 'ready'
    case 'degraded':
    case 'attention':
    case 'collecting':
    case 'queued':
      return 'in-progress'
    case 'offline':
    case 'locked':
      return 'blocked'
    default:
      return status
  }
}

function App() {
  const [snapshot, setSnapshot] = useState<LaunchSnapshot>(fallbackSnapshot)
  const [error, setError] = useState<string | null>(null)
  const [activeSection, setActiveSection] = useState<SectionKey>('overview')
  const [searchQuery, setSearchQuery] = useState('')
  const [queue, setQueue] = useState<QueuedAction[]>([])
  const [queueNotice, setQueueNotice] = useState<string | null>(null)

  useEffect(() => {
    invoke<LaunchSnapshot>('load_control_center_snapshot')
      .then((data) => {
        setSnapshot(data)
        setError(null)
      })
      .catch((err) => {
        setError(String(err))
      })
  }, [])

  const missionTiles = useMemo<MissionTile[]>(() => {
    const onlineNodes = snapshot.nodes.filter((node) => node.status === 'online').length
    const blockedTelemetry = snapshot.telemetry.filter((item) => item.status === 'blocked').length
    const readyReports = snapshot.reports.filter((report) => report.status === 'ready').length
    const vmLaneReady = snapshot.telemetry.some((item) => item.title.toLowerCase().includes('rpc')) && blockedTelemetry === 0

    return [
      {
        title: 'Global readiness',
        value: `${snapshot.overallPercent}%`,
        detail: snapshot.verdict,
        status: snapshot.overallPercent >= 85 ? 'ready' : 'in-progress',
      },
      {
        title: 'Fleet posture',
        value: `${onlineNodes}/${snapshot.nodes.length || 1}`,
        detail: 'online node surfaces',
        status: onlineNodes === snapshot.nodes.length ? 'ready' : 'in-progress',
      },
      {
        title: 'Telemetry guard',
        value: blockedTelemetry === 0 ? 'sealed' : `${blockedTelemetry} risk`,
        detail: 'blocked observability surfaces',
        status: blockedTelemetry === 0 ? 'ready' : 'blocked',
      },
      {
        title: 'Evidence registry',
        value: `${readyReports}/${snapshot.reports.length || 1}`,
        detail: 'release and audit artifacts visible',
        status: readyReports === snapshot.reports.length ? 'ready' : 'in-progress',
      },
      {
        title: 'VM lane seal',
        value: vmLaneReady ? 'sealed' : 'review',
        detail: 'execution lane health for real chain control',
        status: vmLaneReady ? 'ready' : 'in-progress',
      },
    ]
  }, [snapshot])

  const securityGuardrails = useMemo<SecurityGuardrail[]>(() => {
    const hardenedNodes = snapshot.nodes.filter((node) => !node.securityMode.includes('test_fixture')).length
    const lockedWallets = snapshot.wallets.filter((wallet) => wallet.status === 'locked').length
    const missingFiles = snapshot.files.filter((file) => !file.exists).length

    return [
      {
        title: 'Node security profiles',
        value: `${hardenedNodes}/${snapshot.nodes.length || 1}`,
        detail: 'fixture dışı güvenlik modu ile çalışan node yüzeyleri',
        status: hardenedNodes === snapshot.nodes.length ? 'ready' : 'in-progress',
      },
      {
        title: 'Wallet isolation',
        value: `${lockedWallets} cold lane`,
        detail: 'recovery veya air-gapped cüzdan kontrol altında tutuluyor',
        status: lockedWallets > 0 ? 'ready' : 'in-progress',
      },
      {
        title: 'Evidence integrity',
        value: missingFiles === 0 ? 'complete' : `${missingFiles} missing`,
        detail: 'konfigürasyon ve operasyon dosyaları doğrulanıyor',
        status: missingFiles === 0 ? 'ready' : 'blocked',
      },
    ]
  }, [snapshot])

  const integrationLanes = useMemo<IntegrationLane[]>(() => {
    const nodeLane = snapshot.nodes[0]
    const walletLane = snapshot.wallets[0]

    return [
      {
        title: 'Node orchestration lane',
        owner: nodeLane?.role ?? 'node control',
        endpoint: nodeLane?.rpcAddr ?? 'n/a',
        detail: 'Cluster bootstrap, run ve health akışları aynı operasyon katmanında toplanır.',
        status: nodeLane?.status === 'online' ? 'ready' : 'in-progress',
      },
      {
        title: 'Wallet governance lane',
        owner: walletLane?.title ?? 'wallet control',
        endpoint: walletLane?.route ?? 'n/a',
        detail: 'Operator, treasury ve recovery lane politikaları masaüstünden görünür kalır.',
        status: walletLane?.status === 'connected' ? 'ready' : 'in-progress',
      },
      ...snapshot.telemetry.map((item) => ({
        title: item.title,
        owner: 'telemetry surface',
        endpoint: item.target,
        detail: item.detail,
        status: item.status,
      })),
    ]
  }, [snapshot])

  const streamEvents = useMemo<StreamEvent[]>(() => {
    const events: StreamEvent[] = []

    snapshot.blockers.slice(0, 2).forEach((blocker, index) => {
      events.push({
        time: `T-${index + 1}`,
        title: blocker.title,
        detail: blocker.detail,
        severity: 'blocked',
      })
    })

    snapshot.telemetry.slice(0, 2).forEach((item, index) => {
      events.push({
        time: `OBS-${index + 1}`,
        title: item.title,
        detail: `${item.target} · ${item.detail}`,
        severity: item.status === 'blocked' ? 'blocked' : 'in-progress',
      })
    })

    snapshot.reports.slice(0, 1).forEach((report) => {
      events.push({
        time: 'RPT-1',
        title: report.title,
        detail: report.detail,
        severity: report.status === 'ready' ? 'ready' : 'in-progress',
      })
    })

    return events
  }, [snapshot])

  const normalizedQuery = searchQuery.trim().toLowerCase()

  const filteredNodes = useMemo(() => {
    if (!normalizedQuery) return snapshot.nodes
    return snapshot.nodes.filter((node) =>
      [node.id, node.role, node.chainId, node.rpcAddr, node.securityMode].join(' ').toLowerCase().includes(normalizedQuery),
    )
  }, [normalizedQuery, snapshot.nodes])

  const filteredWallets = useMemo(() => {
    if (!normalizedQuery) return snapshot.wallets
    return snapshot.wallets.filter((wallet) =>
      [wallet.title, wallet.route, wallet.addressHint, wallet.detail].join(' ').toLowerCase().includes(normalizedQuery),
    )
  }, [normalizedQuery, snapshot.wallets])

  const filteredRuntime = useMemo(() => {
    const runtime = snapshot.runtimeSurfaces ?? []
    if (!normalizedQuery) return runtime
    return runtime.filter((surface) =>
      [surface.title, surface.target, surface.detail, surface.command].join(' ').toLowerCase().includes(normalizedQuery),
    )
  }, [normalizedQuery, snapshot.runtimeSurfaces])

  const filteredContracts = useMemo(() => {
    const contracts = snapshot.contracts ?? []
    if (!normalizedQuery) return contracts
    return contracts.filter((contract) =>
      [contract.name, contract.lane, contract.vm, contract.address, contract.detail, contract.command]
        .join(' ')
        .toLowerCase()
        .includes(normalizedQuery),
    )
  }, [normalizedQuery, snapshot.contracts])

  const filteredExplorer = useMemo(() => {
    const explorer = snapshot.explorer ?? []
    if (!normalizedQuery) return explorer
    return explorer.filter((surface) =>
      [surface.name, surface.endpoint, surface.detail, surface.command].join(' ').toLowerCase().includes(normalizedQuery),
    )
  }, [normalizedQuery, snapshot.explorer])

  const networkModes = useMemo<NetworkMode[]>(() => {
    const leadNode = snapshot.nodes[0]
    const followerNode = snapshot.nodes[1]
    const observerNode = snapshot.nodes[2]
    return [
      {
        lane: 'devnet',
        chainId: 'aoxc-devnet-local',
        rpc: observerNode?.rpcAddr ?? '127.0.0.1:9545',
        p2p: observerNode?.listenAddr ?? '127.0.0.1:39003',
        bootstrap: 'cargo run -q -p aoxcmd -- devnet-up --profile local-dev',
        status: observerNode?.status === 'offline' ? 'blocked' : 'in-progress',
      },
      {
        lane: 'testnet',
        chainId: leadNode?.chainId ?? 'aoxc-testnet',
        rpc: leadNode?.rpcAddr ?? '127.0.0.1:8545',
        p2p: leadNode?.listenAddr ?? '127.0.0.1:39001',
        bootstrap: 'configs/deterministic-testnet/launch-testnet.sh',
        status: leadNode?.status === 'online' ? 'ready' : 'in-progress',
      },
      {
        lane: 'mainnet',
        chainId: 'aoxc-mainnet-candidate',
        rpc: followerNode?.rpcAddr ?? '127.0.0.1:8546',
        p2p: followerNode?.listenAddr ?? '127.0.0.1:39002',
        bootstrap: 'cargo run -q -p aoxcmd -- production-audit --format json',
        status: snapshot.blockers.length === 0 ? 'ready' : 'in-progress',
      },
    ]
  }, [snapshot.blockers.length, snapshot.nodes])

  const transferScenarios = useMemo<TransferScenario[]>(() => {
    const operatorWallet = snapshot.wallets[0]
    const treasuryWallet = snapshot.wallets.find((wallet) => wallet.title.toLowerCase().includes('treasury'))
    const recoveryWallet = snapshot.wallets.find((wallet) => wallet.title.toLowerCase().includes('recovery'))

    return [
      {
        title: 'Fast validator transfer',
        from: operatorWallet?.addressHint ?? 'AOXC1-VAL-OPER-PRIMARY',
        to: treasuryWallet?.addressHint ?? 'AOXC1-TREASURY-DESKTOP',
        amount: '1250 AOXC',
        feePolicy: 'dynamic priority fee',
        risk: operatorWallet?.status === 'connected' ? 'ready' : 'in-progress',
        command: 'cargo run -q -p aoxcmd -- wallet transfer --profile mainnet --to AOXC1-TREASURY-DESKTOP --amount 1250',
      },
      {
        title: 'Treasury batch transfer',
        from: treasuryWallet?.addressHint ?? 'AOXC1-TREASURY-DESKTOP',
        to: 'AOXC1-MULTI-DEST-BATCH',
        amount: '10000 AOXC / 12 outputs',
        feePolicy: 'policy guarded + co-sign required',
        risk: treasuryWallet?.status === 'attention' ? 'in-progress' : 'ready',
        command: 'cargo run -q -p aoxcmd -- wallet batch-transfer --profile mainnet --plan artifacts/treasury-payouts.json',
      },
      {
        title: 'Recovery emergency bridge',
        from: recoveryWallet?.addressHint ?? 'AOXC1-RECOVERY-ESCROW',
        to: operatorWallet?.addressHint ?? 'AOXC1-VAL-OPER-PRIMARY',
        amount: '250 AOXC',
        feePolicy: 'offline sign + delayed broadcast',
        risk: recoveryWallet?.status === 'locked' ? 'ready' : 'blocked',
        command: 'cargo run -q -p aoxcmd -- wallet emergency-transfer --profile mainnet --dry-run --from recovery',
      },
    ]
  }, [snapshot.wallets])

  const stakeScenarios = useMemo<StakeScenario[]>(() => {
    const onlineNode = snapshot.nodes.find((node) => node.status === 'online')
    const degradedNode = snapshot.nodes.find((node) => node.status !== 'online')
    return [
      {
        title: 'Primary validator staking',
        validator: onlineNode?.id ?? 'atlas',
        strategy: 'long horizon / rewards compounding',
        lock: '21 days',
        estimate: '+9.8% APR (projected)',
        status: 'ready',
        command: 'cargo run -q -p aoxcmd -- stake delegate --validator atlas --amount 5000 --profile mainnet',
      },
      {
        title: 'Risk-balanced staking',
        validator: `${onlineNode?.id ?? 'atlas'} + ${degradedNode?.id ?? 'cypher'}`,
        strategy: 'split stake 70/30 for redundancy',
        lock: '14 days',
        estimate: '+8.7% APR (projected)',
        status: degradedNode ? 'in-progress' : 'ready',
        command: 'cargo run -q -p aoxcmd -- stake rebalance --profile mainnet --policy cautious',
      },
      {
        title: 'Instant unstake drill',
        validator: degradedNode?.id ?? 'cypher',
        strategy: 'incident playbook simulation',
        lock: '0 day (simulation)',
        estimate: 'capital release test',
        status: degradedNode ? 'in-progress' : 'blocked',
        command: 'cargo run -q -p aoxcmd -- stake simulate-unstake --validator cypher --profile testnet',
      },
    ]
  }, [snapshot.nodes])

  const cliLanes = useMemo<CliLane[]>(
    () => [
      {
        title: 'Node & network CLI',
        focus: 'devnet/testnet/mainnet cluster boot and diagnostics',
        commands: [
          ...snapshot.commands,
          {
            title: 'Run single validator',
            command: 'cargo run -q -p aoxcmd -- node-run --home configs/mainnet-validator --rounds 12',
            intent: 'Validator node command pathını bağımsız doğrular.',
          },
          {
            title: 'Chain compatibility matrix',
            command: 'cargo run -q -p aoxcmd -- compat-matrix --format json',
            intent: 'Node + VM lane uyumluluk raporunu üretir.',
          },
        ],
      },
      {
        title: 'Wallet & authority CLI',
        focus: 'wallet address generation, transfer, staking, emergency recovery',
        commands: [
          {
            title: 'Generate operator wallet address',
            command: 'cargo run -q -p aoxcmd -- wallet new-address --profile mainnet --lane operator',
            intent: 'Yeni operatör adresi üretir ve lane metadata kaydeder.',
          },
          {
            title: 'Generate treasury wallet address',
            command: 'cargo run -q -p aoxcmd -- wallet new-address --profile mainnet --lane treasury',
            intent: 'Treasury lane için yeni ödeme adresi üretir.',
          },
          {
            title: 'Inspect wallet safety',
            command: 'cargo run -q -p aoxcmd -- wallet inspect --profile mainnet --verbose',
            intent: 'Signer, policy ve risk durumunu raporlar.',
          },
          ...transferScenarios.map((scenario) => ({
            title: scenario.title,
            command: scenario.command,
            intent: `${scenario.from} → ${scenario.to}`,
          })),
          ...stakeScenarios.map((scenario) => ({
            title: scenario.title,
            command: scenario.command,
            intent: scenario.strategy,
          })),
        ],
      },
      {
        title: 'Reporting & evidence CLI',
        focus: 'audit/report export and operational closure bundle',
        commands: [
          {
            title: 'Desktop ops report (new script)',
            command: 'scripts/generate_desktop_ops_report.sh',
            intent: 'Desktop/UI odaklı günlük operasyon raporu oluşturur.',
          },
          {
            title: 'Network production closure',
            command: 'scripts/validation/network_production_closure.sh',
            intent: 'Mainnet kapanış kanıtlarını paketler.',
          },
          {
            title: 'Release evidence generator',
            command: 'scripts/release/generate_release_evidence.sh',
            intent: 'Release artifact ve audit kanıtlarını üretir.',
          },
        ],
      },
    ],
    [snapshot.commands, stakeScenarios, transferScenarios],
  )

  const explorerSignals = useMemo<ExplorerSignal[]>(() => {
    const healthyExplorer = snapshot.explorer.filter((item) => item.status === 'ready').length
    const riskyExplorer = snapshot.explorer.filter((item) => item.status !== 'ready').length
    const onlineNodes = snapshot.nodes.filter((node) => node.status === 'online').length

    return [
      {
        title: 'Explorer readiness',
        value: `${healthyExplorer}/${snapshot.explorer.length || 1}`,
        detail: 'active explorer surfaces',
        status: riskyExplorer === 0 ? 'ready' : 'in-progress',
      },
      {
        title: 'Live block watchers',
        value: `${onlineNodes}`,
        detail: 'online nodes sending chain updates',
        status: onlineNodes >= 2 ? 'ready' : 'in-progress',
      },
      {
        title: 'Anomaly alerts',
        value: riskyExplorer === 0 ? '0' : `${riskyExplorer}`,
        detail: 'surfaces requiring manual review',
        status: riskyExplorer === 0 ? 'ready' : 'blocked',
      },
    ]
  }, [snapshot.explorer, snapshot.nodes])

  const filteredCommands = useMemo(() => {
    if (!normalizedQuery) return snapshot.commands
    return snapshot.commands.filter((item) =>
      [item.title, item.command, item.intent].join(' ').toLowerCase().includes(normalizedQuery),
    )
  }, [normalizedQuery, snapshot.commands])

  const filteredEvents = useMemo(() => {
    if (!normalizedQuery) return streamEvents
    return streamEvents.filter((event) =>
      [event.title, event.detail, event.time].join(' ').toLowerCase().includes(normalizedQuery),
    )
  }, [normalizedQuery, streamEvents])

  function queueCommand(title: string, command: string, source: string) {
    const action: QueuedAction = {
      id: `${source}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      title,
      command,
      source,
      queuedAt: new Date().toLocaleTimeString(),
    }
    setQueue((current) => [action, ...current].slice(0, 10))
    setQueueNotice(`Queued: ${title}`)
    setTimeout(() => setQueueNotice(null), 1600)
  }

  async function copyCommand(command: string) {
    try {
      await navigator.clipboard.writeText(command)
      setQueueNotice('Command copied')
      setTimeout(() => setQueueNotice(null), 1200)
    } catch {
      setQueueNotice('Clipboard unavailable')
      setTimeout(() => setQueueNotice(null), 1200)
    }
  }

  function renderOverview() {
    return (
      <>
        <section className="mission-grid">
          {missionTiles.map((tile) => (
            <article className="metric-card panel-surface" key={tile.title}>
              <div className="card-topline">
                <span>{tile.title}</span>
                <span className={`status-pill ${tile.status}`}>{statusLabel(tile.status)}</span>
              </div>
              <strong>{tile.value}</strong>
              <p>{tile.detail}</p>
            </article>
          ))}
        </section>

        <section className="dashboard-grid two-col">
          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Readiness tracks</h2>
              <p>Desktop, mainnet ve testnet eksenleri ayrı ayrı izlenir.</p>
            </div>
            <div className="stack-list">
              {snapshot.tracks.map((track) => (
                <article className="info-card compact" key={track.name}>
                  <div className="card-topline">
                    <h3>{track.name}</h3>
                    <span className={`status-pill ${track.status}`}>{statusLabel(track.status as Status)}</span>
                  </div>
                  <strong className="percent">{track.percent}%</strong>
                  <p>{track.summary}</p>
                </article>
              ))}
            </div>
          </article>

          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Live operator stream</h2>
              <p>Blocker, telemetry ve rapor olayları birleşik operatör akışı gibi listelenir.</p>
            </div>
            <div className="timeline-list">
              {filteredEvents.map((event) => (
                <article className={`timeline-item ${event.severity}`} key={`${event.time}-${event.title}`}>
                  <span>{event.time}</span>
                  <strong>{event.title}</strong>
                  <p>{event.detail}</p>
                </article>
              ))}
            </div>
          </article>
        </section>
      </>
    )
  }

  function renderMissionControl() {
    return (
      <section className="dashboard-grid two-col">
        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Open blockers</h2>
            <p>Kapanması gereken release ve operator aksiyonları.</p>
          </div>
          <div className="stack-list">
            {snapshot.blockers.map((blocker) => (
              <article className="info-card compact" key={blocker.title}>
                <div className="card-topline">
                  <h3>{blocker.title}</h3>
                  <span className="status-pill blocked">Blocked</span>
                </div>
                <p>{blocker.detail}</p>
                <code>{blocker.command}</code>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Command presets</h2>
            <p>Desktop arayüzde hızlı operasyon için güvenli komut kaseti.</p>
          </div>
          <div className="stack-list">
            {filteredCommands.map((command) => (
              <article className="info-card compact" key={command.title}>
                <div className="card-topline">
                  <h3>{command.title}</h3>
                  <span className="status-pill ready">Preset</span>
                </div>
                <p>{command.intent}</p>
                <code>{command.command}</code>
                <div className="action-row">
                  <button type="button" onClick={() => queueCommand(command.title, command.command, 'preset')}>
                    Queue
                  </button>
                  <button type="button" onClick={() => copyCommand(command.command)}>
                    Copy
                  </button>
                </div>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Execution queue</h2>
            <p>Desktop üzerinden tetiklenecek zincir/VM/wallet komutları öncelik kuyruğunda tutulur.</p>
          </div>
          <div className="stack-list">
            {queue.length === 0 ? (
              <article className="info-card compact">
                <p className="muted">Queue is empty. Add command presets, node run, or wallet actions.</p>
              </article>
            ) : (
              queue.map((entry) => (
                <article className="info-card compact" key={entry.id}>
                  <div className="card-topline">
                    <h3>{entry.title}</h3>
                    <span className="status-pill in-progress">{entry.source}</span>
                  </div>
                  <p>{entry.queuedAt}</p>
                  <code>{entry.command}</code>
                </article>
              ))
            )}
          </div>
        </article>
      </section>
    )
  }

  function renderSecurity() {
    return (
      <section className="dashboard-grid two-col">
        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Security guardrails</h2>
            <p>Ultra güvenli masaüstü akışı için temel koruma katmanları.</p>
          </div>
          <div className="stack-list">
            {securityGuardrails.map((guardrail) => (
              <article className="info-card compact" key={guardrail.title}>
                <div className="card-topline">
                  <h3>{guardrail.title}</h3>
                  <span className={`status-pill ${guardrail.status}`}>{statusLabel(guardrail.status)}</span>
                </div>
                <strong className="percent">{guardrail.value}</strong>
                <p>{guardrail.detail}</p>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Area readiness</h2>
            <p>Çekirdek alanlar yüzde ve açıklama ile görünür tutulur.</p>
          </div>
          <div className="stack-list">
            {snapshot.areas.map((area) => (
              <article className="info-card compact" key={area.name}>
                <div className="card-topline">
                  <h3>{area.name}</h3>
                  <span className={`status-pill ${area.status}`}>{statusLabel(area.status)}</span>
                </div>
                <strong className="percent">{area.percent}%</strong>
                <p>{area.detail}</p>
              </article>
            ))}
          </div>
        </article>
      </section>
    )
  }

  function renderNetworkModes() {
    return (
      <section className="dashboard-grid two-col">
        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Environment matrix</h2>
            <p>Devnet, testnet ve mainnet lane'leri tek menüde profesyonel şekilde yönetilir.</p>
          </div>
          <div className="stack-list">
            {networkModes.map((mode) => (
              <article className="info-card compact" key={mode.lane}>
                <div className="card-topline">
                  <h3>{mode.lane.toUpperCase()}</h3>
                  <span className={`status-pill ${mode.status}`}>{statusLabel(mode.status)}</span>
                </div>
                <dl className="detail-grid">
                  <div><dt>Chain ID</dt><dd>{mode.chainId}</dd></div>
                  <div><dt>RPC</dt><dd>{mode.rpc}</dd></div>
                  <div><dt>P2P</dt><dd>{mode.p2p}</dd></div>
                </dl>
                <code>{mode.bootstrap}</code>
                <div className="action-row">
                  <button type="button" onClick={() => queueCommand(`${mode.lane} bootstrap`, mode.bootstrap, 'network')}>
                    Queue
                  </button>
                  <button type="button" onClick={() => copyCommand(mode.bootstrap)}>
                    Copy
                  </button>
                </div>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Full CLI command center</h2>
            <p>Tüm önemli CLI komutları UI içinde gruplu, aranabilir ve kuyruklanabilir şekilde listelenir.</p>
          </div>
          <div className="stack-list cli-catalog">
            {cliLanes.map((lane) => (
              <article className="info-card compact cli-lane-card" key={lane.title}>
                <div className="card-topline">
                  <div>
                    <h3>{lane.title}</h3>
                    <p className="muted">{lane.focus}</p>
                  </div>
                  <span className="status-pill ready">{lane.commands.length} cmds</span>
                </div>
                <div className="stack-list">
                  {lane.commands.map((command) => (
                    <article className="info-card compact command-inline" key={`${lane.title}-${command.title}-${command.command}`}>
                      <strong>{command.title}</strong>
                      <p>{command.intent}</p>
                      <code>{command.command}</code>
                      <div className="action-row">
                        <button type="button" onClick={() => queueCommand(command.title, command.command, 'cli-center')}>
                          Queue
                        </button>
                        <button type="button" onClick={() => copyCommand(command.command)}>
                          Copy
                        </button>
                      </div>
                    </article>
                  ))}
                </div>
              </article>
            ))}
          </div>
        </article>
      </section>
    )
  }

  function renderNodes() {
    return (
      <section className="panel-surface section-card">
        <div className="section-heading">
          <h2>Fleet orchestration</h2>
          <p>Node komutları, ağ yüzeyleri ve güvenlik modları gerçek snapshot üzerinden gösterilir.</p>
        </div>
        <div className="stack-list">
          {filteredNodes.map((node) => (
            <article className="info-card compact" key={node.id}>
              <div className="card-topline">
                <div>
                  <h3>{node.id}</h3>
                  <p className="muted">{node.role}</p>
                </div>
                <span className={`status-pill ${statusTone(node.status)}`}>{statusLabel(node.status)}</span>
              </div>
              <dl className="detail-grid">
                <div><dt>Chain</dt><dd>{node.chainId}</dd></div>
                <div><dt>Listen</dt><dd>{node.listenAddr}</dd></div>
                <div><dt>RPC</dt><dd>{node.rpcAddr}</dd></div>
                <div><dt>Peers</dt><dd>{node.peerCount}</dd></div>
                <div><dt>Security</dt><dd>{node.securityMode}</dd></div>
              </dl>
              <code>{node.command}</code>
              <div className="action-row">
                <button type="button" onClick={() => queueCommand(`${node.id} control`, node.command, 'node')}>
                  Queue
                </button>
                <button type="button" onClick={() => copyCommand(node.command)}>
                  Copy
                </button>
              </div>
            </article>
          ))}
        </div>
      </section>
    )
  }

  function renderWallets() {
    return (
      <section className="dashboard-grid two-col">
        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Wallet authority center</h2>
            <p>Operatör, treasury ve recovery lane komutları ve rotalarıyla birlikte izlenir.</p>
          </div>
          <div className="stack-list">
            {filteredWallets.map((wallet) => (
              <article className="info-card compact" key={wallet.title}>
                <div className="card-topline">
                  <h3>{wallet.title}</h3>
                  <span className={`status-pill ${statusTone(wallet.status)}`}>{statusLabel(wallet.status)}</span>
                </div>
                <p>{wallet.detail}</p>
                <dl className="detail-grid">
                  <div><dt>Route</dt><dd>{wallet.route}</dd></div>
                  <div><dt>Address</dt><dd>{wallet.addressHint}</dd></div>
                </dl>
                <code>{wallet.command}</code>
                <div className="action-row">
                  <button type="button" onClick={() => queueCommand(`${wallet.title} action`, wallet.command, 'wallet')}>
                    Queue
                  </button>
                  <button type="button" onClick={() => copyCommand(wallet.command)}>
                    Copy
                  </button>
                </div>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Secure transfer lab</h2>
            <p>Transfer akışları preflight güvenlik kontrolüyle birlikte simüle edilir.</p>
          </div>
          <div className="stack-list">
            {transferScenarios.map((scenario) => (
              <article className="info-card compact wallet-advanced-card" key={scenario.title}>
                <div className="card-topline">
                  <h3>{scenario.title}</h3>
                  <span className={`status-pill ${scenario.risk}`}>{statusLabel(scenario.risk)}</span>
                </div>
                <dl className="detail-grid">
                  <div><dt>From</dt><dd>{scenario.from}</dd></div>
                  <div><dt>To</dt><dd>{scenario.to}</dd></div>
                  <div><dt>Amount</dt><dd>{scenario.amount}</dd></div>
                  <div><dt>Fee policy</dt><dd>{scenario.feePolicy}</dd></div>
                </dl>
                <code>{scenario.command}</code>
                <div className="action-row">
                  <button type="button" onClick={() => queueCommand(`${scenario.title} transfer`, scenario.command, 'wallet-transfer')}>
                    Queue transfer
                  </button>
                  <button type="button" onClick={() => copyCommand(scenario.command)}>
                    Copy
                  </button>
                </div>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Stake strategy vault</h2>
            <p>Stake / unstake / rebalance planları tek merkezde gözden geçirilir.</p>
          </div>
          <div className="stack-list">
            {stakeScenarios.map((scenario) => (
              <article className="info-card compact wallet-advanced-card" key={scenario.title}>
                <div className="card-topline">
                  <h3>{scenario.title}</h3>
                  <span className={`status-pill ${scenario.status}`}>{statusLabel(scenario.status)}</span>
                </div>
                <dl className="detail-grid">
                  <div><dt>Validator</dt><dd>{scenario.validator}</dd></div>
                  <div><dt>Strategy</dt><dd>{scenario.strategy}</dd></div>
                  <div><dt>Lock</dt><dd>{scenario.lock}</dd></div>
                  <div><dt>Yield</dt><dd>{scenario.estimate}</dd></div>
                </dl>
                <code>{scenario.command}</code>
                <div className="action-row">
                  <button type="button" onClick={() => queueCommand(`${scenario.title} stake`, scenario.command, 'wallet-stake')}>
                    Queue stake
                  </button>
                  <button type="button" onClick={() => copyCommand(scenario.command)}>
                    Copy
                  </button>
                </div>
              </article>
            ))}
          </div>
        </article>
      </section>
    )
  }

  function renderTelemetry() {
    return (
      <section className="panel-surface section-card">
        <div className="section-heading">
          <h2>Telemetry surfaces</h2>
          <p>Runtime ve observability yüzeyleri ayrı operatör panelinde gösterilir.</p>
        </div>
        <div className="stack-list">
          {snapshot.telemetry.map((item) => (
            <article className="info-card compact" key={item.title}>
              <div className="card-topline">
                <h3>{item.title}</h3>
                <span className={`status-pill ${statusTone(item.status)}`}>{statusLabel(item.status)}</span>
              </div>
              <p>{item.detail}</p>
              <code>{item.target}</code>
            </article>
          ))}
        </div>
      </section>
    )
  }

  function renderRuntime() {
    return (
      <section className="panel-surface section-card">
        <div className="section-heading">
          <h2>Runtime & RAM control</h2>
          <p>Gerçek node ve VM bellek yüzeyleri ile runtime davranışı masaüstünden yönetilir.</p>
        </div>
        <div className="stack-list">
          {filteredRuntime.map((surface) => (
            <article className="info-card compact" key={surface.title}>
              <div className="card-topline">
                <h3>{surface.title}</h3>
                <span className={`status-pill ${statusTone(surface.status)}`}>{statusLabel(surface.status)}</span>
              </div>
              <dl className="detail-grid">
                <div><dt>Target</dt><dd>{surface.target}</dd></div>
                <div><dt>RAM</dt><dd>{surface.ramMb} MB</dd></div>
              </dl>
              <p>{surface.detail}</p>
              <code>{surface.command}</code>
              <div className="action-row">
                <button type="button" onClick={() => queueCommand(`${surface.title} runtime`, surface.command, 'runtime')}>
                  Queue
                </button>
                <button type="button" onClick={() => copyCommand(surface.command)}>
                  Copy
                </button>
              </div>
            </article>
          ))}
        </div>
      </section>
    )
  }

  function renderContracts() {
    return (
      <section className="panel-surface section-card">
        <div className="section-heading">
          <h2>Contract control center</h2>
          <p>Sistem kontratları, lane durumu ve VM eşleşmesiyle birlikte tek panelde izlenir.</p>
        </div>
        <div className="stack-list">
          {filteredContracts.map((contract) => (
            <article className="info-card compact" key={`${contract.name}-${contract.address}`}>
              <div className="card-topline">
                <h3>{contract.name}</h3>
                <span className={`status-pill ${statusTone(contract.status)}`}>{statusLabel(contract.status)}</span>
              </div>
              <dl className="detail-grid">
                <div><dt>Lane</dt><dd>{contract.lane}</dd></div>
                <div><dt>VM</dt><dd>{contract.vm}</dd></div>
                <div><dt>Address</dt><dd>{contract.address}</dd></div>
              </dl>
              <p>{contract.detail}</p>
              <code>{contract.command}</code>
              <div className="action-row">
                <button type="button" onClick={() => queueCommand(`${contract.name} verify`, contract.command, 'contract')}>
                  Queue
                </button>
                <button type="button" onClick={() => copyCommand(contract.command)}>
                  Copy
                </button>
              </div>
            </article>
          ))}
        </div>
      </section>
    )
  }

  function renderExplorer() {
    return (
      <section className="dashboard-grid two-col">
        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Explorer cockpit</h2>
            <p>Chain state, contract lane ve wallet event explorer yüzeyleri.</p>
          </div>
          <div className="stack-list">
            {filteredExplorer.map((surface) => (
              <article className="info-card compact explorer-advanced-card" key={`${surface.name}-${surface.endpoint}`}>
                <div className="card-topline">
                  <h3>{surface.name}</h3>
                  <span className={`status-pill ${statusTone(surface.status)}`}>{statusLabel(surface.status)}</span>
                </div>
                <p>{surface.detail}</p>
                <dl className="detail-grid">
                  <div><dt>Endpoint</dt><dd>{surface.endpoint}</dd></div>
                </dl>
                <code>{surface.command}</code>
                <div className="action-row">
                  <button type="button" onClick={() => queueCommand(`${surface.name} inspect`, surface.command, 'explorer')}>
                    Queue
                  </button>
                  <button type="button" onClick={() => copyCommand(surface.command)}>
                    Copy
                  </button>
                </div>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Live explorer pulse</h2>
            <p>Explorer metrikleri ile zincir tarama sağlığı tek bakışta görünür.</p>
          </div>
          <div className="mission-grid">
            {explorerSignals.map((signal) => (
              <article className="metric-card panel-surface" key={signal.title}>
                <div className="card-topline">
                  <span>{signal.title}</span>
                  <span className={`status-pill ${signal.status}`}>{statusLabel(signal.status)}</span>
                </div>
                <strong>{signal.value}</strong>
                <p>{signal.detail}</p>
              </article>
            ))}
          </div>
          <article className="info-card compact explorer-timeline panel-surface">
            <h3>Deep scan lanes</h3>
            <ul className="bullet-list">
              {filteredExplorer.map((surface) => (
                <li key={`${surface.name}-scan`}>
                  <strong>{surface.name}</strong>
                  <span>{surface.endpoint}</span>
                </li>
              ))}
            </ul>
          </article>
        </article>
      </section>
    )
  }

  function renderIntegrations() {
    return (
      <section className="dashboard-grid two-col">
        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>System integrations</h2>
            <p>Tüm sistemle entegre çalışan yüzeylerin owner ve endpoint görünürlüğü.</p>
          </div>
          <div className="stack-list">
            {integrationLanes.map((lane) => (
              <article className="info-card compact" key={`${lane.title}-${lane.endpoint}`}>
                <div className="card-topline">
                  <h3>{lane.title}</h3>
                  <span className={`status-pill ${lane.status}`}>{statusLabel(lane.status)}</span>
                </div>
                <dl className="detail-grid single-line">
                  <div><dt>Owner</dt><dd>{lane.owner}</dd></div>
                  <div><dt>Endpoint</dt><dd>{lane.endpoint}</dd></div>
                </dl>
                <p>{lane.detail}</p>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Workspace surfaces</h2>
            <p>AOXChain workspace bileşenleri ve görev sınıfları.</p>
          </div>
          <div className="stack-list">
            {(snapshot.workspaces ?? []).map((workspace) => (
              <article className="info-card compact" key={`${workspace.name}-${workspace.path}`}>
                <div className="card-topline">
                  <h3>{workspace.name}</h3>
                  <span className="status-pill in-progress">{workspace.status}</span>
                </div>
                <p>{workspace.summary}</p>
                <code>{workspace.path}</code>
              </article>
            ))}
          </div>
        </article>
      </section>
    )
  }

  function renderReports() {
    return (
      <section className="dashboard-grid two-col">
        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Reporting & exports</h2>
            <p>Audit, readiness ve closure çıktılarını profesyonel operatör görünümünde toplar.</p>
          </div>
          <div className="stack-list">
            {snapshot.reports.map((report) => (
              <article className="info-card compact" key={report.title}>
                <div className="card-topline">
                  <h3>{report.title}</h3>
                  <span className={`status-pill ${statusTone(report.status)}`}>{statusLabel(report.status)}</span>
                </div>
                <p>{report.detail}</p>
                <code>{report.path}</code>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>AI surfaces</h2>
            <p>Desktop control plane tarafından görülen AI yüzeyleri.</p>
          </div>
          <div className="stack-list">
            {(snapshot.aiSurfaces ?? []).map((surface) => (
              <article className="info-card compact" key={`${surface.name}-${surface.area}`}>
                <div className="card-topline">
                  <h3>{surface.name}</h3>
                  <span className="status-pill in-progress">{surface.status}</span>
                </div>
                <p>{surface.summary}</p>
                <code>{surface.command}</code>
              </article>
            ))}
          </div>
        </article>
      </section>
    )
  }

  function renderEvidence() {
    return (
      <section className="dashboard-grid two-col">
        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Evidence registry</h2>
            <p>Operasyonun güvenli release alabilmesi için gerekli dosya izi.</p>
          </div>
          <div className="stack-list">
            {snapshot.files.map((file) => (
              <article className="info-card compact" key={file.path}>
                <div className="card-topline">
                  <h3>{file.label}</h3>
                  <span className={`status-pill ${file.exists ? 'ready' : 'blocked'}`}>
                    {file.exists ? 'Ready' : 'Missing'}
                  </span>
                </div>
                <code>{file.path}</code>
              </article>
            ))}
          </div>
        </article>

        <article className="panel-surface section-card">
          <div className="section-heading">
            <h2>Operator actions</h2>
            <p>Desktop panelden beklenen en kritik güvenli aksiyonlar.</p>
          </div>
          <div className="action-grid">
            {snapshot.commands.map((command) => (
              <button className="action-tile" type="button" key={command.title}>
                <strong>{command.title}</strong>
                <span>{command.intent}</span>
              </button>
            ))}
          </div>
        </article>
      </section>
    )
  }

  function renderActiveSection() {
    switch (activeSection) {
      case 'overview':
        return renderOverview()
      case 'mission-control':
        return renderMissionControl()
      case 'security':
        return renderSecurity()
      case 'network-modes':
        return renderNetworkModes()
      case 'nodes':
        return renderNodes()
      case 'wallets':
        return renderWallets()
      case 'runtime':
        return renderRuntime()
      case 'contracts':
        return renderContracts()
      case 'explorer':
        return renderExplorer()
      case 'telemetry':
        return renderTelemetry()
      case 'integrations':
        return renderIntegrations()
      case 'reports':
        return renderReports()
      case 'evidence':
        return renderEvidence()
      default:
        return renderOverview()
    }
  }

  const activeLabel = navigation.find((item) => item.key === activeSection)?.label ?? 'Overview'

  return (
    <div className="desktop-shell">
      <aside className="sidebar-shell">
        <div className="brand-block panel-surface">
          <span className="eyebrow">AOXHub desktop</span>
          <h1>Ultra control center</h1>
          <p>Node, wallet, telemetry, kanıt, release ve operasyon kararlarını tek masaüstü arayüzünde yöneten profesyonel cockpit.</p>
        </div>

        <nav className="nav-panel panel-surface">
          <h2>Control lanes</h2>
          <ul className="nav-list">
            {navigation.map((item) => (
              <li key={item.key}>
                <button
                  type="button"
                  className={activeSection === item.key ? 'active' : ''}
                  onClick={() => setActiveSection(item.key)}
                >
                  {item.label}
                </button>
              </li>
            ))}
          </ul>
        </nav>

        <section className="quick-actions panel-surface">
          <div className="section-heading compact-heading">
            <h2>Operator actions</h2>
            <p>Desktop panelden beklenen en kritik güvenli aksiyonlar.</p>
          </div>
          <div className="action-grid">
            {snapshot.commands.map((command) => (
              <button className="action-tile" type="button" key={command.title}>
                <strong>{command.title}</strong>
                <span>{command.intent}</span>
              </button>
            ))}
          </div>
        </section>
      </aside>

      <main className="main-shell">
        <section className="topbar panel-surface">
          <div>
            <span className="eyebrow subtle">{snapshot.profile}</span>
            <h2>{activeLabel}</h2>
            <p>{snapshot.summary}</p>
            <div className="search-row">
              <input
                type="search"
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                placeholder="Search node / wallet / command / evidence..."
                aria-label="Search control center"
              />
              {searchQuery ? (
                <button type="button" onClick={() => setSearchQuery('')}>
                  Clear
                </button>
              ) : null}
            </div>
          </div>
          <div className="topbar-badges">
            <span className={`status-pill ${snapshot.overallPercent >= 85 ? 'ready' : 'in-progress'}`}>{snapshot.stage}</span>
            <span className={`status-pill ${snapshot.blockers.length === 0 ? 'ready' : 'in-progress'}`}>{snapshot.verdict}</span>
            {error ? <span className="status-pill blocked">Fallback snapshot</span> : <span className="status-pill ready">Live repo snapshot</span>}
            {queueNotice ? <span className="status-pill in-progress">{queueNotice}</span> : null}
          </div>
        </section>

        {error ? (
          <section className="panel-surface section-card">
            <div className="section-heading">
              <h2>Snapshot fallback state</h2>
              <p>Repo snapshot okunamadığı için fallback veri gösteriliyor.</p>
            </div>
            <p className="callout warning">{error}</p>
          </section>
        ) : null}

        {renderActiveSection()}
      </main>
    </div>
  )
}

export default App
