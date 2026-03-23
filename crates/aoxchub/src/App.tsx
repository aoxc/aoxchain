import './App.css'

type Track = {
  name: string
  percent: number
  summary: string
  status: 'ready' | 'in-progress'
}

type Blocker = {
  title: string
  action: string
  command: string
}

type FeatureCard = {
  title: string
  status: 'live' | 'next' | 'blocked'
  detail: string
}

const tracks: Track[] = [
  {
    name: 'Mainnet readiness',
    percent: 60,
    summary: 'Production profile, runtime state, operator key, and JSON audit logs must be closed before launch.',
    status: 'in-progress',
  },
  {
    name: 'Testnet readiness',
    percent: 65,
    summary: 'Public testnet is close, but it still needs the same hub and wallet routing discipline to avoid drift.',
    status: 'in-progress',
  },
  {
    name: 'AOXHub parity',
    percent: 100,
    summary: 'Hub mainnet/testnet baseline parity is already recorded in the current readiness evidence.',
    status: 'ready',
  },
]

const blockers: Blocker[] = [
  {
    title: 'Switch to the mainnet production profile',
    action: 'Move the active node profile away from validator mode and enable JSON logs.',
    command: 'aoxc production-bootstrap --profile mainnet --password <value>',
  },
  {
    title: 'Create genesis + runtime state',
    action: 'Materialize committed genesis data and node runtime state for deterministic startup.',
    command: 'aoxc genesis-init && aoxc node-bootstrap',
  },
  {
    title: 'Activate operator keys',
    action: 'Bootstrap or rotate the operator key so signing flows match desktop wallet expectations.',
    command: 'aoxc key-bootstrap --profile mainnet --password <value>',
  },
]

const featureCards: FeatureCard[] = [
  {
    title: 'Desktop wallet routing',
    status: 'live',
    detail: 'Mainnet/testnet routing consistency is already tracked as a release control for AOXHub compatibility.',
  },
  {
    title: 'Hub launch cockpit',
    status: 'next',
    detail: 'This screen turns the current readiness report into an operator-focused launch board inside the desktop app.',
  },
  {
    title: 'One-click launch',
    status: 'blocked',
    detail: 'Should stay blocked until production bootstrap, key activation, and runtime state controls are automated safely.',
  },
]

const milestones = [
  'Close identity blockers: committed genesis material + active operator key.',
  'Close runtime blockers: clean node state bootstrap and startup verification.',
  'Keep hub + wallet parity locked while testnet moves from 65% to launch-ready.',
  'Promote to mainnet only after the remaining weighted controls move from 60% to 100%.',
]

function statusLabel(status: Track['status'] | FeatureCard['status']) {
  switch (status) {
    case 'ready':
    case 'live':
      return 'Ready'
    case 'blocked':
      return 'Blocked'
    default:
      return 'In progress'
  }
}

function App() {
  return (
    <>
      <section className="hero-panel">
        <div className="hero-copy">
          <span className="eyebrow">AOXHub desktop control center</span>
          <h1>Mainnet + testnet + wallet ilerleyişini tek ekranda yönet.</h1>
          <p>
            Hedefin %100 hazır oluş ise önce kalan blocker&apos;ları net kapatmak,
            sonra AOXHub desktop içinde görünür bir launch akışı kurmak gerekiyor.
            Bu arayüz, mevcut repo durumunu ürün diline çeviren başlangıç panelidir.
          </p>
          {error ? <p className="callout warning">Fallback mode: {error}</p> : null}
        </div>

        <div className="hero-summary">
          <div className="summary-ring">
            <strong>60%</strong>
            <span>Mainnet readiness</span>
          </div>
          <ul>
            <li>Testnet: 65%</li>
            <li>AOXHub parity: aligned</li>
            <li>Desktop wallet compatibility evidence: present</li>
          </ul>
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Readiness tracks</h2>
          <p>Mainnet, testnet ve hub hedeflerini aynı anda izleyip sapmayı önleyin.</p>
        </div>
        <div className="track-grid">
          {tracks.map((track) => (
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
            <p>%100 hedefine gitmek için önce bunların kapanması gerekiyor.</p>
          </div>
          <div className="stack-list">
            {blockers.map((blocker) => (
              <article className="info-card compact" key={blocker.title}>
                <h3>{blocker.title}</h3>
                <p>{blocker.action}</p>
                <code>{blocker.command}</code>
              </article>
            ))}
          </div>
        </div>

        <div>
          <div className="section-heading">
            <h2>Desktop product activation</h2>
            <p>Hub + wallet + desktop için ne aktif, ne sırada, ne beklemede gör.</p>
          </div>
          <div className="stack-list">
            {featureCards.map((card) => (
              <article className="info-card compact" key={card.title}>
                <div className="card-topline">
                  <h3>{card.title}</h3>
                  <span className={`status-pill ${card.status}`}>{statusLabel(card.status)}</span>
                </div>
                <p>{card.detail}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <h2>Recommended execution order</h2>
          <p>Full geliştirme için teknik sırayı bozmadan ilerleyelim.</p>
        </div>
        <ol className="milestone-list">
          {milestones.map((milestone) => (
            <li key={milestone}>{milestone}</li>
          ))}
        </ol>
      </section>
    </>
  )
}

export default App
