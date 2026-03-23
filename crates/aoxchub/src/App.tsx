import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

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

type AreaProgress = {
  name: string
  percent: number
  detail: string
  status: 'ready' | 'in-progress'
}

type NodeControl = {
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

type WalletSurface = {
  title: string
  route: string
  status: 'connected' | 'attention' | 'locked'
  addressHint: string
  command: string
  detail: string
}

type TelemetrySurface = {
  title: string
  status: 'ready' | 'blocked'
  target: string
  detail: string
}

type ReportAsset = {
  title: string
  status: 'ready' | 'queued'
  path: string
  detail: string
}

type CommandPreset = {
  title: string
  command: string
  intent: string
}

type ControlCenterSnapshot = {
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

const fallbackSnapshot: ControlCenterSnapshot = {
  stage: 'desktop-control-bootstrap',
  verdict: 'needs-closure',
  overallPercent: 68,
  profile: 'aoxhub.desktop.admin',
  summary:
    'AOXHub admin panel artık zincirin kalbi olacak şekilde node, wallet, telemetry ve raporlama yüzeyini aynı cockpit içinde toplamayı hedefliyor.',
  tracks: [
    {
      name: 'Mainnet readiness',
      percent: 60,
      summary: 'Mainnet profile, structured logs, genesis, node state ve operator key closure gerekiyor.',
      status: 'in-progress',
    },
    {
      name: 'Testnet parity',
      percent: 72,
      summary: 'Desktop yüzeyinin deterministic testnet orchestration ile birebir hizalı olması gerekiyor.',
      status: 'in-progress',
    },
    {
      name: 'Desktop control center',
      percent: 75,
      summary: 'UI iskeleti güçlü ama gerçek komut, wallet ve report adapterları sonraki bağlanacak yüzeyler.',
      status: 'in-progress',
    },
  ],
  blockers: [
    {
      title: 'Three node orchestration',
      detail: 'Desktop panel local/test cluster yönetimini doğrudan CLI/Tauri komutlarına bağlamalı.',
      command: 'configs/deterministic-testnet/launch-testnet.sh',
    },
    {
      title: 'Wallet command bridge',
      detail: 'Wallet inspect/sign/export akışları GUI içinden komut adaptörleriyle yönetilmeli.',
      command: 'aoxc wallet inspect --profile mainnet',
    },
    {
      title: 'Unified reporting',
      detail: 'Launch, telemetry, audit ve forensics aynı export pipeline ile bağlanmalı.',
      command: 'scripts/validation/network_production_closure.sh --scenario soak',
    },
  ],
  files: [
    { label: 'Progress report', path: 'AOXC_PROGRESS_REPORT.md', exists: true },
    { label: 'Mainnet profile', path: 'configs/mainnet.toml', exists: true },
    { label: 'Testnet profile', path: 'configs/testnet.toml', exists: true },
  ],
  areas: [
    { name: 'Configuration', percent: 60, detail: 'Profile ve config drift kapanmalı.', status: 'in-progress' },
    { name: 'Network', percent: 100, detail: 'Peer/baseline kontrolleri hazır.', status: 'ready' },
    { name: 'Observability', percent: 50, detail: 'Structured logging ve telemetry export tamamlanmalı.', status: 'in-progress' },
    { name: 'Identity', percent: 0, detail: 'Genesis ve operator key yüzeyi eksik.', status: 'in-progress' },
  ],
  nodes: [
    {
      id: 'atlas',
      role: 'validator leader',
      status: 'online',
      chainId: 'AOXC-0077-MAIN',
      listenAddr: '127.0.0.1:39001',
      rpcAddr: '127.0.0.1:19101',
      peerCount: 4,
      securityMode: 'mutual_auth_test_fixture',
      command: 'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/atlas --rounds 12 --sleep-ms 200',
    },
    {
      id: 'boreal',
      role: 'validator follower',
      status: 'online',
      chainId: 'AOXC-0077-MAIN',
      listenAddr: '127.0.0.1:39002',
      rpcAddr: '127.0.0.1:19102',
      peerCount: 4,
      securityMode: 'mutual_auth_test_fixture',
      command: 'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/boreal --rounds 12 --sleep-ms 200',
    },
    {
      id: 'cypher',
      role: 'observer / telemetry anchor',
      status: 'degraded',
      chainId: 'AOXC-0077-MAIN',
      listenAddr: '127.0.0.1:39003',
      rpcAddr: '127.0.0.1:19103',
      peerCount: 4,
      securityMode: 'mutual_auth_test_fixture',
      command: 'cargo run -q -p aoxcmd -- node-health --home configs/deterministic-testnet/homes/cypher',
    },
  ],
  wallets: [
    {
      title: 'Operator wallet',
      route: 'mainnet guarded',
      status: 'connected',
      addressHint: 'AOXC1-VAL-OPER-PRIMARY',
      command: 'aoxc key-bootstrap --profile mainnet --password <value>',
      detail: 'Validator lifecycle, governance ve emergency operator eylemleri için ana yüzey.',
    },
    {
      title: 'Treasury wallet',
      route: 'dual-route mainnet/testnet',
      status: 'attention',
      addressHint: 'AOXC1-TREASURY-DESKTOP',
      command: 'aoxc wallet inspect --profile mainnet',
      detail: 'Transfer öncesi policy, audit ve route review gerektirir.',
    },
    {
      title: 'Recovery wallet',
      route: 'offline recovery lane',
      status: 'locked',
      addressHint: 'AOXC1-RECOVERY-ESCROW',
      command: 'aoxc diagnostics-bundle --redact',
      detail: 'Disaster recovery, cold storage ve rotation drill yüzeyi.',
    },
  ],
  telemetry: [
    {
      title: 'Mainnet RPC',
      status: 'ready',
      target: '0.0.0.0:8545',
      detail: 'Mainnet RPC hedefi desktop API/telemetry katmanı için görünür tutulur.',
    },
    {
      title: 'Testnet RPC',
      status: 'ready',
      target: '0.0.0.0:18545',
      detail: 'Testnet route parity ve desktop staging akışları için kullanılır.',
    },
    {
      title: 'Telemetry snapshot',
      status: 'blocked',
      target: 'artifacts/network-production-closure/telemetry-snapshot.json',
      detail: 'Prometheus ve alert evidence burada kapanmalı.',
    },
  ],
  reports: [
    {
      title: 'Release evidence bundle',
      status: 'ready',
      path: 'artifacts/release-evidence',
      detail: 'Compatibility matrix, SBOM ve provenance yüzeyi.',
    },
    {
      title: 'Network production closure',
      status: 'ready',
      path: 'artifacts/network-production-closure',
      detail: 'Soak, telemetry, recovery ve closure bundle burada toplanır.',
    },
    {
      title: 'Progress report',
      status: 'ready',
      path: 'AOXC_PROGRESS_REPORT.md',
      detail: 'Desktop cockpit bu readiness özetinden beslenir.',
    },
  ],
  commands: [
    {
      title: 'Bring up deterministic 3-node cluster',
      command: 'configs/deterministic-testnet/launch-testnet.sh',
      intent: 'Üç node local cluster bootstrap ve orchestration başlangıcı.',
    },
    {
      title: 'Generate production audit',
      command: 'cargo run -q -p aoxcmd -- production-audit --format json',
      intent: 'Operator denetim raporunu GUI reporting katmanına günceller.',
    },
    {
      title: 'Produce closure bundle',
      command: 'scripts/validation/network_production_closure.sh --scenario soak',
      intent: 'Telemetry, runtime ve rollout artefact setini üretir.',
    },
  ],
}

function statusLabel(status: string) {
  switch (status) {
    case 'ready':
    case 'online':
    case 'connected':
      return 'Ready'
    case 'in-progress':
    case 'degraded':
    case 'attention':
    case 'queued':
    case 'collecting':
      return 'In progress'
    default:
      return 'Blocked'
  }
}

function App() {
  const [snapshot, setSnapshot] = useState<ControlCenterSnapshot>(fallbackSnapshot)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    invoke<ControlCenterSnapshot>('load_control_center_snapshot')
      .then((data) => {
        setSnapshot(data)
        setError(null)
      })
      .catch((err) => setError(String(err)))
  }, [])

  const headlineStats = useMemo(
    () => [
      { label: 'Overall readiness', value: `${snapshot.overallPercent}%`, detail: snapshot.verdict },
      { label: 'Desktop profile', value: snapshot.profile, detail: snapshot.stage },
      { label: 'Node surfaces', value: `${snapshot.nodes.length}`, detail: 'Cluster control surfaces' },
      { label: 'Wallet lanes', value: `${snapshot.wallets.length}`, detail: 'Operator + treasury + recovery' },
    ],
    [snapshot],
  )

  return (
    <>
      <section className="hero-panel">
        <div className="hero-copy">
          <span className="eyebrow">AOXHub system heart</span>
          <h1>Wallet, node yönetimi, telemetry ve full raporlama tek desktop panelde.</h1>
          <p>
            Bu sürüm, AOXHub’ı sadece güzel bir dashboard olmaktan çıkarıp zincirin kontrol merkezi
            olacak yöne taşıyor. Hedef; operator, wallet, telemetry, node orchestration ve audit
            yüzeylerini aynı masaüstü cockpit içinde toplamak.
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
          <h2>Chain control modules</h2>
          <p>Zincirin kalbi olarak panelin hangi alanlara odaklandığını readiness bazlı gösterir.</p>
        </div>
        <div className="module-grid">
          {snapshot.areas.map((area) => (
            <article className="info-card panel-surface" key={area.name}>
              <div className="card-topline">
                <h3>{area.name}</h3>
                <span className={`status-pill ${area.status}`}>{statusLabel(area.status)}</span>
              </div>
              <strong className="percent">{area.percent}%</strong>
              <p>{area.detail}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Readiness tracks</h2>
          <p>Mainnet, testnet ve desktop control-center hedeflerini aynı yerde tut.</p>
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
            <h2>3-node control plane</h2>
            <p>Deterministic testnet node yüzeyleri artık backend snapshot’tan okunuyor.</p>
          </div>
          <div className="stack-list">
            {snapshot.nodes.map((node) => (
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
                    <dt>Chain</dt>
                    <dd>{node.chainId}</dd>
                  </div>
                  <div>
                    <dt>Listen</dt>
                    <dd>{node.listenAddr}</dd>
                  </div>
                  <div>
                    <dt>RPC</dt>
                    <dd>{node.rpcAddr}</dd>
                  </div>
                  <div>
                    <dt>Peers</dt>
                    <dd>{node.peerCount}</dd>
                  </div>
                </dl>
                <div className="inline-command">
                  <span>{node.securityMode}</span>
                  <code>{node.command}</code>
                </div>
              </article>
            ))}
          </div>
        </div>

        <div>
          <div className="section-heading">
            <h2>Wallet command center</h2>
            <p>Wallet yüzeyleri artık panelin ayrı modülü değil, doğrudan operasyon kalbi.</p>
          </div>
          <div className="stack-list">
            {snapshot.wallets.map((wallet) => (
              <article className="info-card compact panel-surface" key={wallet.title}>
                <div className="card-topline">
                  <h3>{wallet.title}</h3>
                  <span className={`status-pill ${wallet.status}`}>{statusLabel(wallet.status)}</span>
                </div>
                <p>{wallet.detail}</p>
                <dl className="detail-grid">
                  <div>
                    <dt>Route</dt>
                    <dd>{wallet.route}</dd>
                  </div>
                  <div>
                    <dt>Address</dt>
                    <dd>{wallet.addressHint}</dd>
                  </div>
                </dl>
                <div className="inline-command">
                  <span>Command bridge</span>
                  <code>{wallet.command}</code>
                </div>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Telemetry and API surfaces</h2>
          <p>Gerçek node yönetimi için hangi RPC/telemetry hedeflerinin panelde görüldüğü netleşiyor.</p>
        </div>
        <div className="track-grid">
          {snapshot.telemetry.map((stream) => (
            <article className="info-card panel-surface" key={stream.title}>
              <div className="card-topline">
                <h3>{stream.title}</h3>
                <span className={`status-pill ${stream.status}`}>{statusLabel(stream.status)}</span>
              </div>
              <code>{stream.target}</code>
              <p>{stream.detail}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-block split-layout split-layout-wide">
        <div>
          <div className="section-heading">
            <h2>Reporting center</h2>
            <p>Launch, telemetry, recovery ve release evidence burada görünür.</p>
          </div>
          <div className="stack-list">
            {snapshot.reports.map((report) => (
              <article className="info-card compact panel-surface" key={report.title}>
                <div className="card-topline">
                  <h3>{report.title}</h3>
                  <span className={`status-pill ${report.status}`}>{statusLabel(report.status)}</span>
                </div>
                <p>{report.detail}</p>
                <code>{report.path}</code>
              </article>
            ))}
          </div>
        </div>

        <div>
          <div className="section-heading">
            <h2>Operator command queue</h2>
            <p>Bir sonraki Tauri adapter katmanına bağlanacak gerçek komut presetleri.</p>
          </div>
          <div className="stack-list">
            {snapshot.commands.map((item) => (
              <article className="info-card compact panel-surface" key={item.title}>
                <h3>{item.title}</h3>
                <code>{item.command}</code>
                <p>{item.intent}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section-block split-layout">
        <div>
          <div className="section-heading">
            <h2>Launch blockers</h2>
            <p>%100 kapanış için desktop panelde açık kalması gereken gerçek engeller.</p>
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
            <p>Panelin gerçekten gördüğü config/runbook/report dosyaları.</p>
          </div>
          <div className="stack-list">
            {snapshot.files.map((file) => (
              <article className="info-card compact panel-surface" key={file.path}>
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
        </div>
      </section>
    </>
  )
}

export default App
