const items = [
  'Overview',
  'Nodes',
  'Wallets',
  'Telemetry',
  'Explorer',
  'Reports',
  'Databases',
  'Logs',
  'Terminals',
  'Evidence',
]

export function Sidebar() {
  return (
    <aside className="app-sidebar panel-surface">
      <h2>Operations map</h2>
      <nav>
        <ul className="sidebar-menu">
          {items.map((item) => (
            <li key={item}>
              <button type="button">{item}</button>
            </li>
          ))}
        </ul>
      </nav>
    </aside>
  )
}
