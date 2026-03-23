import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

type Status = 'ready' | 'in-progress' | 'blocked' | 'online' | 'degraded' | 'offline' | 'connected' | 'attention' | 'locked' | 'collecting' | 'queued'

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

const fallbackSnapshot: LaunchSnapshot = {
  stage: 'Desktop operations baseline',
  verdict: 'Needs operator closure',
  overallPercent: 68,
  profile: 'aoxhub.desktop.admin',
  summary: 'AOXHub masaüstü omurgası; node, wallet, telemetry, audit ve release kanıtlarını tek kontrol merkezinde toplar.',
  tracks: [
    {
      name: 'Mainnet readiness',
      percent: 60,
      summary: 'Production kontrollleri ve release kanıtları kapanmalı.',
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
      command: 'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/atlas --rounds 12 --sleep-ms 200',
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
      command: 'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/boreal --rounds 12 --sleep-ms 200',
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
      command: 'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/cypher --rounds 12 --sleep-ms 200',
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
}

const navItems = [
  'Overview',
  'Mission control',
  'Security',
  'Nodes',
  'Wallets',
  'Telemetry',
  'Integrations',
  'Reports',
  'Evidence',
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
            {navItems.map((item) => (
              <li key={item}>
                <button type="button">{item}</button>
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
            <h2>Tüm sistemi tek noktadan yöneten tam entegre desktop operator plane</h2>
            <p>{snapshot.summary}</p>
          </div>
          <div className="topbar-badges">
            <span className={`status-pill ${snapshot.overallPercent >= 85 ? 'ready' : 'in-progress'}`}>{snapshot.stage}</span>
            <span className={`status-pill ${snapshot.blockers.length === 0 ? 'ready' : 'in-progress'}`}>{snapshot.verdict}</span>
            {error ? <span className="status-pill blocked">Fallback snapshot</span> : <span className="status-pill ready">Live repo snapshot</span>}
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
            <h2>Gerçek repo verileriyle beslenen güvenli node + wallet + telemetry + evidence kokpiti</h2>
            <p>
              Bu yüzey sadece görsel bir dashboard değil; doğrudan AOXChain repo durumundan snapshot çekip komut presetleri,
              güvenlik katmanları, node erişim yüzeyleri ve kanıt envanteriyle birleşik bir operator experience sunar.
            </p>
            {error ? <p className="callout warning">Tauri snapshot alınamadı, fallback veri gösteriliyor: {error}</p> : null}
            <div className="command-strip">
              {snapshot.commands.map((item) => (
                <article key={item.title} className="command-card">
                  <span>{item.title}</span>
                  <code>{item.command}</code>
                  <small>{item.intent}</small>
                </article>
              ))}
            </div>
          </div>
          <div className="hero-side-grid">
            <article className="focus-card panel-surface">
              <span className="muted">Primary objective</span>
              <strong>Mainnet, cüzdan yönetişimi, audit ve kanıt akışlarını operatör için tek profesyonel masaüstü yüzeyinde toplamak</strong>
            </article>
            <article className="focus-card panel-surface">
              <span className="muted">Open blockers</span>
              <ul className="bullet-list">
                {snapshot.blockers.map((blocker) => (
                  <li key={blocker.title}>
                    <strong>{blocker.title}</strong>
                    <span>{blocker.detail}</span>
                    <code>{blocker.command}</code>
                  </li>
                ))}
              </ul>
            </article>
          </div>
        </section>

        <section className="dashboard-grid three-col">
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

          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Live operator stream</h2>
              <p>Blocker, telemetry ve rapor olayları birleşik operatör akışı gibi listelenir.</p>
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
        </section>

        <section className="dashboard-grid two-col">
          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Fleet orchestration</h2>
              <p>Node komutları, ağ yüzeyleri ve güvenlik modları gerçek snapshot üzerinden gösterilir.</p>
            </div>
            <div className="stack-list">
              {snapshot.nodes.map((node) => (
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
                </article>
              ))}
            </div>
          </article>

          <article className="panel-surface section-card">
            <div className="section-heading">
              <h2>Wallet authority center</h2>
              <p>Operatör, treasury ve recovery lane komutları ve rotalarıyla birlikte izlenir.</p>
            </div>
            <div className="stack-list">
              {snapshot.wallets.map((wallet) => (
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
                </article>
              ))}
            </div>
          </article>
        </section>

        <section className="dashboard-grid three-col">
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
        </section>

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
              <h2>Command presets</h2>
              <p>Desktop arayüzde hızlı operasyon için güvenli komut kaseti.</p>
            </div>
            <div className="stack-list">
              {snapshot.commands.map((command) => (
                <article className="info-card compact" key={command.title}>
                  <div className="card-topline">
                    <h3>{command.title}</h3>
                    <span className="status-pill ready">Preset</span>
                  </div>
                  <p>{command.intent}</p>
                  <code>{command.command}</code>
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
