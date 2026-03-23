import type { CommandPreset, ReportAsset, TerminalSurface, FileStatus, LaunchBlocker } from '../../types/controlCenter'

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

export function OperationsSection({ reports, commands, terminals, blockers, files }: { reports: ReportAsset[]; commands: CommandPreset[]; terminals: TerminalSurface[]; blockers: LaunchBlocker[]; files: FileStatus[] }) {
  return (
    <>
      <section className="section-block split-layout split-layout-wide">
        <div>
          <div className="section-heading"><h2>Reporting center</h2><p>Release, closure ve progress raporları ayrı kartlarda.</p></div>
          <div className="stack-list">
            {reports.map((report) => (
              <article className="info-card compact panel-surface" key={report.title}>
                <div className="card-topline"><h3>{report.title}</h3><span className={`status-pill ${report.status}`}>{statusLabel(report.status)}</span></div>
                <p>{report.detail}</p>
                <code>{report.path}</code>
              </article>
            ))}
          </div>
        </div>
        <div>
          <div className="section-heading"><h2>Operator commands</h2><p>Komut presetleri UI butonlarına bağlanmaya hazır.</p></div>
          <div className="stack-list">
            {commands.map((item) => (
              <article className="info-card compact panel-surface" key={item.title}>
                <h3>{item.title}</h3>
                <code>{item.command}</code>
                <p>{item.intent}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section-block split-layout split-layout-wide">
        <div>
          <div className="section-heading"><h2>Terminal rail</h2><p>Üstte/ayrık terminal mantığı için komut rayları.</p></div>
          <div className="stack-list">
            {terminals.map((terminal) => (
              <article className="info-card compact panel-surface" key={terminal.title}>
                <h3>{terminal.title}</h3>
                <code>{terminal.command}</code>
                <p>{terminal.detail}</p>
              </article>
            ))}
          </div>
        </div>
        <div>
          <div className="section-heading"><h2>Blockers & evidence</h2><p>Engeller ve görülen dosyalar aynı operasyon railinde.</p></div>
          <div className="stack-list">
            {blockers.map((blocker) => (
              <article className="info-card compact panel-surface" key={blocker.title}>
                <h3>{blocker.title}</h3>
                <p>{blocker.detail}</p>
                <code>{blocker.command}</code>
              </article>
            ))}
            {files.map((file) => (
              <article className="info-card compact panel-surface" key={file.path}>
                <div className="card-topline"><h3>{file.label}</h3><span className={`status-pill ${file.exists ? 'ready' : 'blocked'}`}>{file.exists ? 'Ready' : 'Missing'}</span></div>
                <code>{file.path}</code>
              </article>
            ))}
          </div>
        </div>
      </section>
    </>
  )
}
