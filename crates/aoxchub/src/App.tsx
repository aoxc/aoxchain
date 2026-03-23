import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

type LaunchSnapshot = {
  stage: string
  verdict: string
  overallPercent: number
  profile: string
  summary: string
  tracks: Track[]
  blockers: LaunchBlocker[]
  files: FileStatus[]
}

type Track = {
  name: string
  percent: number
  summary: string
  status: 'ready' | 'in-progress'
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

type NodeStatus = 'online' | 'degraded' | 'offline'
type WalletState = 'connected' | 'attention' | 'locked'
type ReportState = 'ready' | 'collecting' | 'queued'
type Severity = 'critical' | 'warning' | 'stable'

type DesktopNode = {
  id: string
  role: string
  zone: string
  status: NodeStatus
  rpc: string
  latestHeight: string
  peers: number
  sync: string
  latency: string
  action: string
}

type WalletPanel = {
  title: string
  address: string
  network: string
  state: WalletState
  balance: string
  approvals: string
  detail: string
}

type ReportCard = {
  title: string
  state: ReportState
  description: string
  output: string
  cadence: string
}

type ActionCommand = {
  title: string
  command: string
  outcome: string
}

type MissionTile = {
  title: string
  value: string
  detail: string
  status: 'ready' | 'in-progress' | 'blocked'
}

type StreamEvent = {
  time: string
  title: string
  detail: string
  severity: Severity
}

type ServicePanel = {
  title: string
  owner: string
  uptime: string
  status: 'ready' | 'in-progress' | 'blocked'
  detail: string
}

type QuickAction = {
  label: string
  hint: string
}

const fallbackSnapshot: LaunchSnapshot = {
  stage: 'Desktop operations baseline',
  verdict: 'Needs operator closure',
  overallPercent: 68,
  profile: 'aoxhub.desktop.admin',
  summary:
    'AOXHub masaüstü omurgası; node, wallet, audit, explorer ve raporlama akışını tek bir yönetim kokpitinde toplar.',
  tracks: [
    {
      name: 'Mainnet control plane',
      percent: 60,
      summary: 'Bootstrap, validator rotasyonu, güvenlik ve release kanıtları kapanmalı.',
      status: 'in-progress',
    },
    {
      name: 'Treasury & wallet governance',
      percent: 73,
      summary: 'Approval zinciri, çoklu imza görünürlüğü ve transfer guardrail katmanları ilerliyor.',
      status: 'in-progress',
    },
    {
      name: 'Desktop command center',
      percent: 82,
      summary: 'Masaüstü cockpit, operatörün günlük yönetim ihtiyaçlarını kapsayacak seviyeye geldi.',
      status: 'ready',
    },
  ],
  blockers: [
    {
      title: 'Three-node orchestrator wiring',
      detail: 'Validator + observer cluster akışları güvenli komut adaptörleri üzerinden finalize edilmeli.',
      command: 'aoxc cluster up --profile testnet --nodes 3 --with-observer',
    },
    {
      title: 'Wallet signing guardrails',
      detail: 'Transfer, imza ve release onayları için çevre bazlı policy görünürlüğü UI içinde netleşmeli.',
      command: 'aoxc wallet review --scope desktop --export json',
    },
    {
      title: 'Unified evidence export',
      detail: 'Health, audit, logs ve readiness kanıtları tek paket halinde arşivlenmeli.',
      command: 'aoxc ops report --include nodes,wallet,audit,release --format html',
    },
  ],
  files: [
    { label: 'Progress report', path: 'AOXC_PROGRESS_REPORT.md', exists: true },
    { label: 'Mainnet profile', path: 'configs/mainnet.toml', exists: true },
    { label: 'Testnet profile', path: 'configs/testnet.toml', exists: true },
    { label: 'AOXHub mainnet profile', path: 'configs/aoxhub-mainnet.toml', exists: true },
    { label: 'AOXHub testnet profile', path: 'configs/aoxhub-testnet.toml', exists: true },
  ],
}

const desktopNodes: DesktopNode[] = [
  {
    id: 'node-01',
    role: 'Validator leader',
    zone: 'Mainnet secure lane',
    status: 'online',
    rpc: '127.0.0.1:8545',
    latestHeight: '1,882,410',
    peers: 18,
    sync: '99.98%',
    latency: '41 ms',
    action: 'Rotate key + rolling restart',
  },
  {
    id: 'node-02',
    role: 'Quorum backup',
    zone: 'Mainnet fallback lane',
    status: 'online',
    rpc: '127.0.0.1:8546',
    latestHeight: '1,882,407',
    peers: 17,
    sync: '99.94%',
    latency: '44 ms',
    action: 'Replay audit bundle',
  },
  {
    id: 'node-03',
    role: 'Observer & analytics',
    zone: 'Forensics / reporting lane',
    status: 'degraded',
    rpc: '127.0.0.1:9545',
    latestHeight: '1,882,395',
    peers: 11,
    sync: '99.22%',
    latency: '103 ms',
    action: 'Rebuild snapshot cache',
  },
]

const walletPanels: WalletPanel[] = [
  {
    title: 'Operator wallet',
    address: 'AOXC1-VAL-OPER-9JK3',
    network: 'Mainnet guarded',
    state: 'connected',
    balance: '48,220 AOXC',
    approvals: '2/2 active',
    detail: 'Validator ücretleri, governance imzaları ve operasyon fonları için ana kasa yüzeyi.',
  },
  {
    title: 'Treasury wallet',
    address: 'AOXC1-TRE-OPS-2PL8',
    network: 'Dual route',
    state: 'attention',
    balance: '5,100 AOXC',
    approvals: '2/3 pending',
    detail: 'Çıkış policy raporu eksik; deploy öncesi çoklu onay ve limit kontrolü gerekli.',
  },
  {
    title: 'Recovery wallet',
    address: 'AOXC1-REC-DR-7MN1',
    network: 'Offline recovery lane',
    state: 'locked',
    balance: 'Cold storage',
    approvals: 'air-gapped',
    detail: 'Acil durum anahtar rotasyonu ve felaket kurtarma tatbikatı için kilitli tutulur.',
  },
]

const reportCards: ReportCard[] = [
  {
    title: 'Launch readiness report',
    state: 'ready',
    description: 'Mainnet, testnet, hub ve wallet kapanış durumunu tek yönetici dosyasında özetler.',
    output: 'reports/launch-readiness.json',
    cadence: 'Every 15 min',
  },
  {
    title: 'Node forensic bundle',
    state: 'collecting',
    description: 'Node logları, peer sapmaları, restart geçmişi ve snapshot hashlerini toplar.',
    output: 'reports/node-forensics.tar.zst',
    cadence: 'Streaming',
  },
  {
    title: 'Wallet audit ledger',
    state: 'queued',
    description: 'İmza geçmişi, policy approvals ve export aktivitelerini ledger formatında toplar.',
    output: 'reports/wallet-audit.ndjson',
    cadence: 'Queued by trigger',
  },
]

const commandQueue: ActionCommand[] = [
  {
    title: '3-node cluster bootstrap',
    command: 'aoxc cluster up --profile testnet --nodes 3 --with-observer',
    outcome: 'Validator çifti ve observer düğümünü tek yönetim akışında ayağa kaldırır.',
  },
  {
    title: 'Wallet security review',
    command: 'aoxc wallet review --scope desktop --export json',
    outcome: 'İmza politikası, key-state ve release-route sapmalarını denetler.',
  },
  {
    title: 'Unified evidence pack',
    command: 'aoxc ops report --include nodes,wallet,audit,release --format html',
    outcome: 'Tüm operasyon kanıtlarını yöneticiye hazır tek pakete dönüştürür.',
  },
]

const streamEvents: StreamEvent[] = [
  {
    time: '09:42 UTC',
    title: 'Observer node drift detected',
    detail: 'node-03 snapshot cache yeniden inşa edilmeli; explorer tile etkileniyor.',
    severity: 'warning',
  },
  {
    time: '09:31 UTC',
    title: 'Treasury approval window open',
    detail: 'İkinci imza mevcut; üçüncü approver beklemede.',
    severity: 'critical',
  },
  {
    time: '09:18 UTC',
    title: 'Readiness bundle exported',
    detail: 'Launch readiness raporu başarıyla güncellendi.',
    severity: 'stable',
  },
]

const servicePanels: ServicePanel[] = [
  {
    title: 'Explorer & chain scan',
    owner: 'Read API',
    uptime: '99.94%',
    status: 'ready',
    detail: 'Blok, işlem, adres ve governance akışları aynı veri katmanından sunulur.',
  },
  {
    title: 'Telemetry & alerts',
    owner: 'Metrics bus',
    uptime: '99.68%',
    status: 'in-progress',
    detail: 'Node gecikmeleri, sync sapmaları ve policy ihlalleri için alarm üretir.',
  },
  {
    title: 'Database & backups',
    owner: 'Ops storage',
    uptime: '99.71%',
    status: 'in-progress',
    detail: 'Snapshot, log ve audit verileri için sıcak/soğuk saklama katmanını yönetir.',
  },
]

const quickActions: QuickAction[] = [
  { label: 'Node failover', hint: 'Leader fallback akışını tetikle' },
  { label: 'Safe transfer', hint: 'Policy kontrollü ödeme başlat' },
  { label: 'Snapshot verify', hint: 'Hash ve yükseklik doğrula' },
  { label: 'Export evidence', hint: 'Hazır rapor paketi üret' },
]

function statusLabel(status: Track['status'] | NodeStatus | WalletState | ReportState | MissionTile['status']) {
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

function App() {
  const [snapshot, setSnapshot] = useState<LaunchSnapshot>(fallbackSnapshot)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    invoke<LaunchSnapshot>('load_launch_snapshot')
      .then((data) => {
        setSnapshot(data)
        setError(null)
      })
      .catch((err) => {
        setError(String(err))
      })
  }, [])

  const missionTiles = useMemo<MissionTile[]>(
    () => [
      {
        title: 'Global readiness',
        value: `${snapshot.overallPercent}%`,
        detail: snapshot.verdict,
        status: snapshot.overallPercent > 79 ? 'ready' : 'in-progress',
      },
      {
        title: 'Node fleet',
        value: `${desktopNodes.filter((node) => node.status === 'online').length}/${desktopNodes.length}`,
        detail: 'online cluster members',
        status: desktopNodes.some((node) => node.status === 'degraded') ? 'in-progress' : 'ready',
      },
      {
        title: 'Wallet approvals',
        value: walletPanels[1]?.approvals ?? 'n/a',
        detail: 'treasury transfer guardrail',
        status: 'in-progress',
      },
      {
        title: 'Evidence files',
        value: `${snapshot.files.filter((file) => file.exists).length}/${snapshot.files.length}`,
        detail: 'visible release artifacts',
        status: snapshot.files.every((file) => file.exists) ? 'ready' : 'blocked',
      },
    ],
    [snapshot],
  )

  return (
    <div className="desktop-shell">
      <aside className="sidebar-shell">
        <div className="brand-block panel-surface">
          <span className="eyebrow">AOXHub desktop</span>
          <h1>Ultra ops dashboard</h1>
          <p>Zincirin tamamını tek masaüstü cockpit üzerinden yöneten ileri seviye yüzey.</p>
        </div>

        <nav className="nav-panel panel-surface">
          <h2>Command lanes</h2>
          <ul className="nav-list">
            {['Overview', 'Mission control', 'Nodes', 'Wallets', 'Explorer', 'Telemetry', 'Reports', 'Terminal', 'Evidence'].map((item) => (
              <li key={item}><button type="button">{item}</button></li>
            ))}
          </ul>
        </nav>

        <section className="quick-actions panel-surface">
          <div className="section-heading compact-heading">
            <h2>Quick actions</h2>
            <p>Operatörün en hızlı erişmesi gereken kısayollar.</p>
          </div>
          <div className="action-grid">
            {quickActions.map((action) => (
              <button className="action-tile" type="button" key={action.label}>
                <strong>{action.label}</strong>
                <span>{action.hint}</span>
              </button>
            ))}
          </div>
        </section>
      </aside>

      <main className="main-shell">
        <section className="topbar panel-surface">
          <div>
            <span className="eyebrow subtle">{snapshot.profile}</span>
            <h2>%100 sistem yönetimi için tasarlanmış birleşik operasyon dashboard’u</h2>
            <p>{snapshot.summary}</p>
          </div>
          <div className="topbar-badges">
            <span className="status-pill ready">{snapshot.stage}</span>
            <span className="status-pill in-progress">{snapshot.verdict}</span>
            {error ? <span className="status-pill blocked">Fallback mode</span> : <span className="status-pill ready">Live snapshot</span>}
          </div>
        </section>

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

        <section className="hero-command panel-surface">
          <div className="hero-command-copy">
            <span className="eyebrow">Mission control</span>
            <h2>Node, wallet, telemetry, explorer, kanıt ve terminal katmanları tek ekranda.</h2>
            <p>
              Bu tasarım klasik “status page” değil; doğrudan yönetim, gözlem, müdahale ve raporlama akışlarını
              aynı masaüstü deneyiminde birleştiren gerçek bir operasyon merkezi.
            </p>
            {error ? <p className="callout warning">Fallback mode: {error}</p> : null}
            <div className="command-strip">
              {commandQueue.map((item) => (
                <article key={item.title} className="command-card">
                  <span>{item.title}</span>
                  <code>{item.command}</code>
                  <small>{item.outcome}</small>
                </article>
              ))}
            </div>
          </div>
          <div className="hero-side-grid">
            <article className="focus-card panel-surface">
              <span className="muted">Primary objective</span>
              <strong>Mainnet + treasury + audit kapanışını operatör için görünür ve müdahale edilebilir yapmak</strong>
            </article>
            <article className="focus-card panel-surface">
              <span className="muted">Active blockers</span>
              <ul className="bullet-list">
                {snapshot.blockers.map((blocker) => (
                  <li key={blocker.title}>
                    <strong>{blocker.title}</strong>
                    <span>{blocker.detail}</span>
                  </li>
                ))}
              </ul>
            </article>
          </div>
        </section>

        <section className="dashboard-grid two-col">
          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>3-node fleet control</h2>
              <p>Validator, backup ve observer düğümleri ayrı izlenir, aynı anda yönetilir.</p>
            </div>
            <div className="stack-list">
              {desktopNodes.map((node) => (
                <article className="info-card compact" key={node.id}>
                  <div className="card-topline">
                    <div>
                      <h3>{node.id}</h3>
                      <p className="muted">{node.role}</p>
                    </div>
                    <span className={`status-pill ${node.status}`}>{statusLabel(node.status)}</span>
                  </div>
                  <p className="muted">{node.zone}</p>
                  <dl className="detail-grid">
                    <div><dt>RPC</dt><dd>{node.rpc}</dd></div>
                    <div><dt>Height</dt><dd>{node.latestHeight}</dd></div>
                    <div><dt>Peers</dt><dd>{node.peers}</dd></div>
                    <div><dt>Sync</dt><dd>{node.sync}</dd></div>
                    <div><dt>Latency</dt><dd>{node.latency}</dd></div>
                    <div><dt>Action</dt><dd>{node.action}</dd></div>
                  </dl>
                </article>
              ))}
            </div>
          </article>

          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Wallet authority center</h2>
              <p>Operatör, treasury ve recovery cüzdanları görev ve güvenlik bazlı ayrılır.</p>
            </div>
            <div className="stack-list">
              {walletPanels.map((wallet) => (
                <article className="info-card compact" key={wallet.title}>
                  <div className="card-topline">
                    <h3>{wallet.title}</h3>
                    <span className={`status-pill ${wallet.state}`}>{statusLabel(wallet.state)}</span>
                  </div>
                  <p>{wallet.detail}</p>
                  <dl className="detail-grid">
                    <div><dt>Address</dt><dd>{wallet.address}</dd></div>
                    <div><dt>Network</dt><dd>{wallet.network}</dd></div>
                    <div><dt>Balance</dt><dd>{wallet.balance}</dd></div>
                    <div><dt>Approvals</dt><dd>{wallet.approvals}</dd></div>
                  </dl>
                </article>
              ))}
            </div>
          </article>
        </section>

        <section className="dashboard-grid three-col">
          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Readiness tracks</h2>
              <p>Operasyonun hangi eksende ne kadar ilerlediği sürekli görünür kalır.</p>
            </div>
            <div className="stack-list">
              {snapshot.tracks.map((track) => (
                <article className="info-card compact" key={track.name}>
                  <div className="card-topline">
                    <h3>{track.name}</h3>
                    <span className={`status-pill ${track.status}`}>{statusLabel(track.status)}</span>
                  </div>
                  <strong className="percent">{track.percent}%</strong>
                  <p>{track.summary}</p>
                </article>
              ))}
            </div>
          </article>

          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Live event stream</h2>
              <p>Alarm, onay ve export olayları gerçek zaman hissi verecek şekilde listelenir.</p>
            </div>
            <div className="timeline-list">
              {streamEvents.map((event) => (
                <article className={`timeline-item ${event.severity}`} key={`${event.time}-${event.title}`}>
                  <span>{event.time}</span>
                  <strong>{event.title}</strong>
                  <p>{event.detail}</p>
                </article>
              ))}
            </div>
          </article>

          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Service mesh</h2>
              <p>Explorer, telemetry ve storage katmanları operasyon hizmeti olarak görünür.</p>
            </div>
            <div className="stack-list">
              {servicePanels.map((service) => (
                <article className="info-card compact" key={service.title}>
                  <div className="card-topline">
                    <h3>{service.title}</h3>
                    <span className={`status-pill ${service.status}`}>{statusLabel(service.status)}</span>
                  </div>
                  <dl className="detail-grid single-line">
                    <div><dt>Owner</dt><dd>{service.owner}</dd></div>
                    <div><dt>Uptime</dt><dd>{service.uptime}</dd></div>
                  </dl>
                  <p>{service.detail}</p>
                </article>
              ))}
            </div>
          </article>
        </section>

        <section className="dashboard-grid two-col">
          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Reporting & exports</h2>
              <p>Raporlar sadece liste değil; üretim sıklığı ve hedef çıktısıyla birlikte gösterilir.</p>
            </div>
            <div className="stack-list">
              {reportCards.map((report) => (
                <article className="info-card compact" key={report.title}>
                  <div className="card-topline">
                    <h3>{report.title}</h3>
                    <span className={`status-pill ${report.state}`}>{statusLabel(report.state)}</span>
                  </div>
                  <p>{report.description}</p>
                  <dl className="detail-grid single-line">
                    <div><dt>Output</dt><dd>{report.output}</dd></div>
                    <div><dt>Cadence</dt><dd>{report.cadence}</dd></div>
                  </dl>
                </article>
              ))}
            </div>
          </article>

          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Evidence registry</h2>
              <p>Operasyon kanıtı ve konfigürasyon dosyaları merkezi bir kayıt defterinde izlenir.</p>
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
        </section>
      </main>
    </div>
  )
}

export default App
