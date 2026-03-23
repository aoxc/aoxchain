import { useCallback, useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

type ReadinessTrack = {
  name: string
  percent: number
  summary: string
  status: 'ready' | 'in-progress'
}

type AreaProgress = {
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

type LaunchSnapshot = {
  stage: string
  verdict: string
  overallPercent: number
  profile: string
  summary: string
  lastRefreshedAt: number
  tracks: ReadinessTrack[]
  areaProgress: AreaProgress[]
  recommendedFocus: string[]
  remediationPlan: string[]
  blockers: LaunchBlocker[]
  files: FileStatus[]
}

const fallbackSnapshot: LaunchSnapshot = {
  stage: 'integration-hardening',
  verdict: 'not-ready',
  overallPercent: 60,
  profile: 'validator',
  summary: 'Live repo snapshot could not be loaded, so AOXHub is showing the embedded baseline.',
  lastRefreshedAt: 0,
  tracks: [
    {
      name: 'Mainnet readiness',
      percent: 60,
      summary: 'Production profile, runtime state, operator key, and JSON audit logs still need closure.',
      status: 'in-progress',
    },
    {
      name: 'Testnet readiness',
      percent: 65,
      summary: 'Public testnet is close, but it still needs the same hub and wallet routing discipline.',
      status: 'in-progress',
    },
    {
      name: 'Overall program',
      percent: 60,
      summary: 'AOXHub should only move to launch mode after the repo-backed readiness controls reach 100%.',
      status: 'in-progress',
    },
  ],
  areaProgress: [
    { name: 'configuration', percent: 60, summary: 'in-progress', status: 'in-progress' },
    { name: 'network', percent: 100, summary: 'ready', status: 'ready' },
    { name: 'identity', percent: 0, summary: 'bootstrap', status: 'in-progress' },
  ],
  recommendedFocus: [
    'identity: raise from 0% to 100% (0 of 2 checks passing)',
    'runtime: raise from 0% to 100% (0 of 1 checks passing)',
  ],
  remediationPlan: [
    'Run aoxc production-bootstrap --profile mainnet --password <value> or aoxc config-init --profile mainnet --json-logs.',
  ],
  blockers: [
    {
      title: 'Mainnet profile',
      detail: 'Active profile is still validator.',
      command: 'aoxc production-bootstrap --profile mainnet --password <value>',
    },
    {
      title: 'Structured logging',
      detail: 'JSON logs are required for audit trails and SIEM ingestion.',
      command: 'aoxc config-init --profile mainnet --json-logs',
    },
  ],
  files: [
    { label: 'Progress report', path: 'AOXC_PROGRESS_REPORT.md', exists: true },
    { label: 'Mainnet profile', path: 'configs/mainnet.toml', exists: true },
    { label: 'Testnet profile', path: 'configs/testnet.toml', exists: true },
  ],
}

function statusLabel(status: ReadinessTrack['status']) {
  return status === 'ready' ? 'Ready' : 'In progress'
}

function formatRefreshTime(timestamp: number) {
  if (!timestamp) {
    return 'Embedded fallback data'
  }

  return new Date(timestamp * 1000).toLocaleString('en-US', {
    dateStyle: 'medium',
    timeStyle: 'short',
  })
}

function App() {
  const [snapshot, setSnapshot] = useState<LaunchSnapshot>(fallbackSnapshot)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)

  const loadSnapshot = useCallback(async () => {
    setRefreshing(true)
    try {
      const result = await invoke<LaunchSnapshot>('load_launch_snapshot')
      setSnapshot(result)
      setError(null)
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason))
    } finally {
      setLoading(false)
      setRefreshing(false)
    }
  }, [])

  useEffect(() => {
    void loadSnapshot()
  }, [loadSnapshot])

  const completedTracks = useMemo(
    () => snapshot.tracks.filter((track) => track.status === 'ready').length,
    [snapshot.tracks],
  )

  const remainingToTarget = Math.max(0, 100 - snapshot.overallPercent)
  const screenDevelopmentPercent = useMemo(() => {
    const checks = [
      snapshot.tracks.length > 0,
      snapshot.areaProgress.length > 0,
      snapshot.blockers.length > 0,
      snapshot.files.length > 0,
      snapshot.recommendedFocus.length > 0,
      snapshot.remediationPlan.length > 0,
      snapshot.lastRefreshedAt > 0,
    ]

    return Math.round((checks.filter(Boolean).length / checks.length) * 100)
  }, [snapshot])

  return (
    <>
      <section className="hero-panel">
        <div className="hero-copy">
          <span className="eyebrow">AOXHub desktop control center</span>
          <h1>Mainnet, testnet ve wallet akışını gerçek repo verisiyle yönet.</h1>
          <p>
            AOXHub artık sadece readiness yüzdesi göstermiyor; area progress,
            recommended focus, remediation plan ve kritik dosya görünürlüğünü de tek
            ekrana topluyor. Böylece full geliştirim sırasında hangi eksik kaç puan
            etkiliyor daha net izlenebiliyor.
          </p>
          <div className="hero-actions">
            <button className="action-button" onClick={() => void loadSnapshot()} disabled={refreshing}>
              {refreshing ? 'Refreshing…' : 'Refresh snapshot'}
            </button>
            <span className="refresh-meta">Last refresh: {formatRefreshTime(snapshot.lastRefreshedAt)}</span>
          </div>
          {error ? <p className="callout warning">Fallback mode: {error}</p> : null}
        </div>

        <div className="hero-summary">
          <div className="summary-ring">
            <strong>{snapshot.overallPercent}%</strong>
            <span>{snapshot.verdict}</span>
          </div>
          <ul>
            <li>Stage: {snapshot.stage}</li>
            <li>Profile: {snapshot.profile}</li>
            <li>Tracks ready: {completedTracks}/{snapshot.tracks.length}</li>
            <li>Open blockers: {snapshot.blockers.length}</li>
            <li>Remaining to 100%: {remainingToTarget}%</li>
            <li>Desktop screen maturity: {screenDevelopmentPercent}%</li>
          </ul>
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Live readiness tracks</h2>
          <p>{snapshot.summary}</p>
        </div>
        <div className="track-grid">
          {snapshot.tracks.map((track) => (
            <article className="info-card" key={track.name}>
              <div className="card-topline">
                <span>{track.name}</span>
                <span className={`status-pill ${track.status}`}>{statusLabel(track.status)}</span>
              </div>
              <strong className="percent">{track.percent}%</strong>
              <div className="progress-bar" aria-hidden="true">
                <span style={{ width: `${track.percent}%` }}></span>
              </div>
              <p>{track.summary}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-block split-layout">
        <div>
          <div className="section-heading">
            <h2>Area progress</h2>
            <p>Readiness raporundaki alt alanlar artık ayrı ayrı izlenebiliyor.</p>
          </div>
          <div className="stack-list">
            {snapshot.areaProgress.map((area) => (
              <article className="info-card compact" key={area.name}>
                <div className="card-topline">
                  <h3>{area.name}</h3>
                  <span className={`status-pill ${area.status}`}>{area.percent}%</span>
                </div>
                <div className="progress-bar" aria-hidden="true">
                  <span style={{ width: `${area.percent}%` }}></span>
                </div>
                <p>{area.summary}</p>
              </article>
            ))}
          </div>
        </div>

        <div>
          <div className="section-heading">
            <h2>Launch blockers</h2>
            <p>Report dosyasındaki kalan blocker&apos;lar direkt burada listelenir.</p>
          </div>
          <div className="stack-list">
            {snapshot.blockers.map((blocker) => (
              <article className="info-card compact" key={blocker.title}>
                <h3>{blocker.title}</h3>
                <p>{blocker.detail}</p>
                <code>{blocker.command}</code>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section-block split-layout">
        <div>
          <div className="section-heading">
            <h2>Recommended focus</h2>
            <p>Bir sonraki sprintte en çok etki edecek alanları burada topla.</p>
          </div>
          <ul className="bullet-list">
            {snapshot.recommendedFocus.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </div>

        <div>
          <div className="section-heading">
            <h2>Remediation plan</h2>
            <p>Kapanış için önerilen eylemler direkt rapordan taşınır.</p>
          </div>
          <ul className="bullet-list">
            {snapshot.remediationPlan.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Config + evidence files</h2>
          <p>Desktop, hub ve network aktivasyonu için kritik dosyalar görünür durumda.</p>
        </div>
        <div className="track-grid file-grid">
          {snapshot.files.map((file) => (
            <article className="info-card compact" key={file.path}>
              <div className="card-topline">
                <h3>{file.label}</h3>
                <span className={`status-pill ${file.exists ? 'ready' : 'blocked'}`}>
                  {file.exists ? 'Present' : 'Missing'}
                </span>
              </div>
              <code>{file.path}</code>
            </article>
          ))}
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Current mode</h2>
          <p>
            {loading
              ? 'Launch snapshot yükleniyor...'
              : `AOXHub desktop ekranı şu anda ${screenDevelopmentPercent}% ürünleşmiş durumda.`}
          </p>
        </div>
      </section>
    </>
  )
}

export default App
