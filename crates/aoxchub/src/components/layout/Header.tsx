import type { ControlCenterSnapshot } from '../../types/controlCenter'

export function Header({ snapshot }: { snapshot: ControlCenterSnapshot }) {
  return (
    <header className="app-header">
      <div>
        <span className="eyebrow">AOXHub ultra control center</span>
        <h1>Wallet + node + telemetry + explorer + terminal aynı operasyon omurgasında.</h1>
      </div>
      <div className="header-badges">
        <span className="status-pill in-progress">{snapshot.profile}</span>
        <span className="status-pill ready">{snapshot.stage}</span>
      </div>
    </header>
  )
}
