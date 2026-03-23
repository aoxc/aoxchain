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

type DesktopNode = {
  id: string
  role: string
  status: NodeStatus
  rpc: string
  latestHeight: string
  peers: number
  sync: string
  action: string
}

type WalletPanel = {
  title: string
  address: string
  network: string
  state: WalletState
  balance: string
  detail: string
}

type ReportCard = {
  title: string
  state: ReportState
  description: string
  output: string
}

type ActionCommand = {
  title: string
  command: string
  outcome: string
}

const fallbackSnapshot: LaunchSnapshot = {
  stage: 'Desktop operations baseline',
  verdict: 'Needs operator closure',
  overallPercent: 68,
  profile: 'aoxhub.desktop.admin',
  summary:
    'Desktop admin panel hedefi güçlü, ama production kapanış için orchestrated node flows, wallet controls ve reporting evidence aynı yüzeyde birleştirilmeli.',
  tracks: [
    {
      name: 'Mainnet control plane',
      percent: 60,
      summary: 'Mainnet rollout için bootstrap, key activation, state integrity ve release evidence kapanmalı.',
      status: 'in-progress',
    },
    {
      name: 'Testnet parity',
      percent: 72,
      summary: 'Testnet tarafı masaüstü cockpit ile parity tutuyor ama otomasyon zinciri henüz tam değil.',
      status: 'in-progress',
    },
    {
      name: 'Desktop surface',
      percent: 74,
      summary: 'Wallet + node + reporting aynı ekranda birleşmeye başladı; komut entegrasyonları sonraki aşama.',
      status: 'in-progress',
    },
  ],
  blockers: [
    {
      title: 'Three-node orchestrator wiring',
      detail: 'Desktop panel, 3 düğümlü local/test cluster akışını güvenli komut adapterlarıyla bağlamalı.',
      command: 'aoxc cluster up --profile testnet --nodes 3',
    },
    {
      title: 'Wallet signing guardrails',
      detail: 'Desktop wallet, imza/transfer akışlarında environment ve release profile ayrımını görünür tutmalı.',
      command: 'aoxc wallet inspect --profile mainnet',
    },
    {
      title: 'Unified reporting export',
      detail: 'Health, launch readiness, logs ve audit evidence tek rapor paketine bağlanmalı.',
      command: 'aoxc ops report --format json --include wallet,node,audit',
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
    role: 'Validator / leader candidate',
    status: 'online',
    rpc: '127.0.0.1:8545',
    latestHeight: '1,882,410',
    peers: 18,
    sync: '99.98%',
    action: 'Rotate key + restart',
  },
  {
    id: 'node-02',
    role: 'Validator / quorum backup',
    status: 'online',
    rpc: '127.0.0.1:8546',
    latestHeight: '1,882,407',
    peers: 17,
    sync: '99.94%',
    action: 'Replay audit bundle',
  },
  {
    id: 'node-03',
    role: 'Observer / reporting anchor',
    status: 'degraded',
    rpc: '127.0.0.1:9545',
    latestHeight: '1,882,395',
    peers: 11,
    sync: '99.22%',
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
    detail: 'Validator fee, rotation ve governance imzaları için ana masaüstü kasa yüzeyi.',
  },
  {
    title: 'Treasury wallet',
    address: 'AOXC1-TRE-OPS-2PL8',
    network: 'Dual-route mainnet/testnet',
    state: 'attention',
    balance: '5,100 AOXC',
    detail: 'Outgoing policy ve multi-approver raporu eksik; deploy öncesi review gerekli.',
  },
  {
    title: 'Recovery wallet',
    address: 'AOXC1-REC-DR-7MN1',
    network: 'Offline recovery lane',
    state: 'locked',
    balance: 'Cold storage',
    detail: 'Emergency rotation ve DR drill tetiklemek için kasıtlı olarak kilitli tutuluyor.',
  },
]

const reportCards: ReportCard[] = [
  {
    title: 'Launch readiness report',
    state: 'ready',
    description: 'Mainnet/testnet/hub/wallet kapanış durumunu tek pakette özetler.',
    output: 'reports/launch-readiness.json',
  },
  {
    title: 'Node forensic bundle',
    state: 'collecting',
    description: '3 düğüm logları, peer sapmaları, restart geçmişi ve snapshot hashleri.',
    output: 'reports/node-forensics.tar.zst',
  },
  {
    title: 'Wallet + audit ledger',
    state: 'queued',
    description: 'Desktop wallet imzaları, policy approvals ve export geçmişi.',
    output: 'reports/wallet-audit.ndjson',
  },
]

const commandQueue: ActionCommand[] = [
  {
    title: 'Bring up 3-node local cluster',
    command: 'aoxc cluster up --profile testnet --nodes 3 --with-observer',
    outcome: 'Bootstraps validator pair + observer/reporting node from the desktop runbook.',
  },
  {
    title: 'Open wallet security review',
    command: 'aoxc wallet review --scope desktop --export json',
    outcome: 'Collects signing posture, key-state and release-route mismatches before transfer approval.',
  },
  {
    title: 'Generate unified closure report',
    command: 'aoxc ops report --include nodes,wallet,audit,release --format html',
    outcome: 'Produces the operator-facing report pack the admin panel is designed to surface.',
  },
]

function statusLabel(status: Track['status'] | NodeStatus | WalletState | ReportState) {
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

  const headlineStats = useMemo(
    () => [
      { label: 'Overall readiness', value: `${snapshot.overallPercent}%`, detail: snapshot.verdict },
      { label: 'Desktop profile', value: snapshot.profile, detail: snapshot.stage },
      { label: 'Active blockers', value: `${snapshot.blockers.length}`, detail: 'Launch-gate items remain visible' },
      { label: 'Report assets', value: `${reportCards.length}`, detail: 'Wallet + node + audit bundles' },
    ],
    [snapshot],
  )

  return (
    <>
      <section className="hero-panel">
        <div className="hero-copy">
          <span className="eyebrow">AOXHub desktop admin panel</span>
          <h1>Desktop GUI wallet + 3 node cockpit + full raporlama omurgası.</h1>
          <p>
            Evet, zinciri sadece metin raporuyla bitirmek zor. Bu yüzden AOXHub yüzeyini,
            operatörün node, wallet, readiness ve audit akışını aynı masaüstü panelinden
            yönetebileceği ileri seviye admin cockpit yönüne taşıdım.
          </p>
          <p className="hero-note">{snapshot.summary}</p>
          {error ? <p className="callout warning">Fallback mode: {error}</p> : null}
        </div>

        <div className="hero-summary panel-surface">
          <div className="summary-ring">
            <strong>{snapshot.overallPercent}%</strong>
            <span>{snapshot.verdict}</span>
          </div>
          <div className="summary-meta">
            <div>
              <span>Stage</span>
              <strong>{snapshot.stage}</strong>
            </div>
            <div>
              <span>Profile</span>
              <strong>{snapshot.profile}</strong>
            </div>
          </div>
          <div className="stat-strip">
            {headlineStats.map((stat) => (
              <article key={stat.label} className="stat-chip">
                <span>{stat.label}</span>
                <strong>{stat.value}</strong>
                <small>{stat.detail}</small>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Admin cockpit modules</h2>
          <p>Bu ekran artık sadece readiness özeti değil; desktop operasyon yüzeyinin modül haritası.</p>
        </div>
        <div className="module-grid">
          {[
            ['Launch control', 'Mainnet/testnet promotion, blocker closure ve release verdict görünümü.'],
            ['3-node orchestrator', 'Validator çift + observer node için tek ekranda durum ve aksiyon akışı.'],
            ['Desktop wallet', 'Operatör, treasury ve recovery wallet yüzeyleri; route ve güvenlik görünürlüğü.'],
            ['Reporting hub', 'Health, audit, logs, forensic bundle ve export dosyaları aynı panelde.'],
          ].map(([title, description]) => (
            <article className="info-card panel-surface" key={title}>
              <h3>{title}</h3>
              <p>{description}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Readiness tracks</h2>
          <p>Mainnet, testnet ve desktop yönetim hedeflerini tek yerde tut.</p>
        </div>
        <div className="track-grid">
          {snapshot.tracks.map((track) => (
            <article className="info-card panel-surface" key={track.name}>
              <div className="card-topline">
                <span>{track.name}</span>
                <span className={`status-pill ${track.status}`}>{statusLabel(track.status)}</span>
              </div>
              <strong className="percent">{track.percent}%</strong>
              <p>{track.summary}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-block split-layout split-layout-wide">
        <div>
          <div className="section-heading">
            <h2>3-node cluster control</h2>
            <p>Validator + observer kurulumunu masaüstünden yönetmek için başlangıç kontrol yüzeyi.</p>
          </div>
          <div className="stack-list">
            {desktopNodes.map((node) => (
              <article className="info-card compact panel-surface" key={node.id}>
                <div className="card-topline">
                  <div>
                    <h3>{node.id}</h3>
                    <p className="muted">{node.role}</p>
                  </div>
                  <span className={`status-pill ${node.status}`}>{statusLabel(node.status)}</span>
                </div>
                <dl className="detail-grid">
                  <div>
                    <dt>RPC</dt>
                    <dd>{node.rpc}</dd>
                  </div>
                  <div>
                    <dt>Height</dt>
                    <dd>{node.latestHeight}</dd>
                  </div>
                  <div>
                    <dt>Peers</dt>
                    <dd>{node.peers}</dd>
                  </div>
                  <div>
                    <dt>Sync</dt>
                    <dd>{node.sync}</dd>
                  </div>
                </dl>
                <div className="inline-command">
                  <span>Suggested action</span>
                  <code>{node.action}</code>
                </div>
              </article>
            ))}
          </div>
        </div>

        <div>
          <div className="section-heading">
            <h2>Desktop wallet center</h2>
            <p>Wallet dahil demiştin; bu blok operatör/wallet yüzeyini admin panelin içine alıyor.</p>
          </div>
          <div className="stack-list">
            {walletPanels.map((wallet) => (
              <article className="info-card compact panel-surface" key={wallet.title}>
                <div className="card-topline">
                  <h3>{wallet.title}</h3>
                  <span className={`status-pill ${wallet.state}`}>{statusLabel(wallet.state)}</span>
                </div>
                <p>{wallet.detail}</p>
                <dl className="detail-grid">
                  <div>
                    <dt>Address</dt>
                    <dd>{wallet.address}</dd>
                  </div>
                  <div>
                    <dt>Network</dt>
                    <dd>{wallet.network}</dd>
                  </div>
                  <div>
                    <dt>Balance</dt>
                    <dd>{wallet.balance}</dd>
                  </div>
                </dl>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section-block split-layout split-layout-wide">
        <div>
          <div className="section-heading">
            <h2>Full reporting center</h2>
            <p>Health, audit, forensic ve export çıktılarını aynı yerden topla.</p>
          </div>
          <div className="stack-list">
            {reportCards.map((report) => (
              <article className="info-card compact panel-surface" key={report.title}>
                <div className="card-topline">
                  <h3>{report.title}</h3>
                  <span className={`status-pill ${report.state}`}>{statusLabel(report.state)}</span>
                </div>
                <p>{report.description}</p>
                <code>{report.output}</code>
              </article>
            ))}
          </div>
        </div>

        <div>
          <div className="section-heading">
            <h2>Operator command queue</h2>
            <p>GUI içinden tetiklenecek bir sonraki doğal komut adapterları burada tanımlı.</p>
          </div>
          <div className="stack-list">
            {commandQueue.map((item) => (
              <article className="info-card compact panel-surface" key={item.title}>
                <h3>{item.title}</h3>
                <code>{item.command}</code>
                <p>{item.outcome}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section-block split-layout">
        <div>
          <div className="section-heading">
            <h2>Launch blockers</h2>
            <p>%100 kapanış için desktop panelde görünür kalması gereken işler.</p>
          </div>
          <div className="stack-list">
            {snapshot.blockers.map((blocker) => (
              <article className="info-card compact panel-surface" key={blocker.title}>
                <h3>{blocker.title}</h3>
                <p>{blocker.detail}</p>
                <code>{blocker.command}</code>
              </article>
            ))}
          </div>
        </div>

        <div>
          <div className="section-heading">
            <h2>Evidence surface</h2>
            <p>Panelin hangi kaynak dosyaları gördüğünü açıkça göster.</p>
          </div>
          <div className="stack-list">
            {snapshot.files.map((file) => (
              <article className="info-card compact panel-surface" key={file.path}>
                <div className="card-topline">
                  <h3>{file.label}</h3>
                  <span className={`status-pill ${file.exists ? 'ready' : 'locked'}`}>
                    {file.exists ? 'Ready' : 'Missing'}
                  </span>
                </div>
                <code>{file.path}</code>
              </article>
            ))}
          </div>
        </div>
      </section>
    </>
  )
}

export default App
