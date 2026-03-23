import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

type ReadinessTrack = {
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
  tracks: ReadinessTrack[]
  blockers: LaunchBlocker[]
  files: FileStatus[]
}

const fallbackSnapshot: LaunchSnapshot = {
  stage: 'integration-hardening',
  verdict: 'not-ready',
  overallPercent: 60,
  profile: 'validator',
  summary: 'Live repo snapshot could not be loaded, so AOXHub is showing the embedded baseline.',
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

function App() {
  const [snapshot, setSnapshot] = useState<LaunchSnapshot>(fallbackSnapshot)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let active = true

    invoke<LaunchSnapshot>('load_launch_snapshot')
      .then((result) => {
        if (!active) return
        setSnapshot(result)
        setError(null)
      })
      .catch((reason) => {
        if (!active) return
        setError(reason instanceof Error ? reason.message : String(reason))
      })
      .finally(() => {
        if (active) setLoading(false)
      })

    return () => {
      active = false
    }
  }, [])

  const completedTracks = useMemo(
    () => snapshot.tracks.filter((track) => track.status === 'ready').length,
    [snapshot.tracks],
  )

  return (
    <>
      <section className="hero-panel">
        <div className="hero-copy">
          <span className="eyebrow">AOXHub desktop control center</span>
          <h1>Mainnet, testnet ve wallet akışını gerçek repo verisiyle yönet.</h1>
          <p>
            Statik demo yerine artık AOXHub desktop, repo içindeki readiness raporunu
            okuyup kalan blocker&apos;ları ve konfigürasyon dosyalarını tek ekranda
            gösteriyor. Böylece %100 hedefe giderken neyin eksik olduğu daha net.
          </p>
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
              <p>{track.summary}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-block split-layout">
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

        <div>
          <div className="section-heading">
            <h2>Config + evidence files</h2>
            <p>Desktop, hub ve network aktivasyonu için kritik dosyalar görünür durumda.</p>
          </div>
          <div className="stack-list">
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
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Current mode</h2>
          <p>
            {loading
              ? 'Launch snapshot yükleniyor...'
              : 'AOXHub artık desktop içinde gerçek readiness bilgisini göstermeye hazır.'}
          </p>
        </div>
      </section>
    </>
  )
}

export default App
