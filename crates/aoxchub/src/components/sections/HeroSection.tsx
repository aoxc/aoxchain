import type { ControlCenterSnapshot } from '../../types/controlCenter'

export function HeroSection({ snapshot, error }: { snapshot: ControlCenterSnapshot; error: string | null }) {
  const stats = [
    ['Overall readiness', `${snapshot.overallPercent}%`, snapshot.verdict],
    ['Node surfaces', `${snapshot.nodes.length}`, 'cluster control planes'],
    ['Wallet lanes', `${snapshot.wallets.length}`, 'operator / treasury / recovery'],
    ['Explorer views', `${snapshot.explorer.length}`, 'progress / fixtures / artifacts'],
  ]

  return (
    <section className="hero-panel">
      <div className="hero-copy">
        <p>{snapshot.summary}</p>
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
          {stats.map(([label, value, detail]) => (
            <article key={label} className="stat-chip">
              <span>{label}</span>
              <strong>{value}</strong>
              <small>{detail}</small>
            </article>
          ))}
        </div>
      </div>
    </section>
  )
}
