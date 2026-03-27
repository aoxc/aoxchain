// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

import type { ReactNode } from 'react'
import type { AreaProgress, Track, TelemetrySurface, ExplorerSurface, DatabaseSurface, LogSurface } from '../../types/controlCenter'

type GridItem = AreaProgress | Track | TelemetrySurface | ExplorerSurface | DatabaseSurface | LogSurface

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

export function SurfaceGrid({ title, description, items, getBody }: { title: string; description: string; items: GridItem[]; getBody: (item: GridItem) => ReactNode }) {
  return (
    <section className="section-block">
      <div className="section-heading">
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      <div className="track-grid">
        {items.map((item) => (
          <article className="info-card panel-surface" key={'title' in item ? item.title : item.name}>
            <div className="card-topline">
              <h3>{'title' in item ? item.title : item.name}</h3>
              <span className={`status-pill ${item.status}`}>{statusLabel(item.status)}</span>
            </div>
            {getBody(item)}
          </article>
        ))}
      </div>
    </section>
  )
}
