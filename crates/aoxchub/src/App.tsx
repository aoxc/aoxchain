import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { fallbackSnapshot } from './data/fallbackSnapshot'
import { Footer } from './components/layout/Footer'
import { Header } from './components/layout/Header'
import { Sidebar } from './components/layout/Sidebar'
import { HeroSection } from './components/sections/HeroSection'
import { NodeWalletSection } from './components/sections/NodeWalletSection'
import { OperationsSection } from './components/sections/OperationsSection'
import { SurfaceGrid } from './components/sections/SurfaceGrid'
import type { ControlCenterSnapshot } from './types/controlCenter'
import './App.css'

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

  return (
    <div className="app-shell">
      <Header snapshot={snapshot} />
      <div className="app-body">
        <Sidebar />
        <main className="app-main">
          <HeroSection snapshot={snapshot} error={error} />
          <SurfaceGrid
            title="Chain control modules"
            description="Zincirin kalbi olarak panelin hangi alanlara odaklandığını readiness bazlı gösterir."
            items={snapshot.areas}
            getBody={(area) => (
              <>
                <strong className="percent">{'percent' in area ? area.percent : 0}%</strong>
                <p>{'detail' in area ? area.detail : ''}</p>
              </>
            )}
          />
          <SurfaceGrid
            title="Readiness tracks"
            description="Mainnet, testnet ve desktop control-center hedeflerini aynı yerde tut."
            items={snapshot.tracks}
            getBody={(track) => (
              <>
                <strong className="percent">{'percent' in track ? track.percent : 0}%</strong>
                <p>{'summary' in track ? track.summary : ''}</p>
              </>
            )}
          />
          <NodeWalletSection nodes={snapshot.nodes} wallets={snapshot.wallets} />
          <SurfaceGrid
            title="Telemetry and API surfaces"
            description="RPC ve telemetry hedeflerini bağımsız sayfa yükü gibi ayırır."
            items={snapshot.telemetry}
            getBody={(stream) => (
              <>
                <code>{'target' in stream ? stream.target : ''}</code>
                <p>{'detail' in stream ? stream.detail : ''}</p>
              </>
            )}
          />
          <SurfaceGrid
            title="Explorer surfaces"
            description="Explorer, fixture, artifact ve veri gezgini kartları ayrı tutulur."
            items={snapshot.explorer}
            getBody={(item) => (
              <>
                <code>{'target' in item ? item.target : ''}</code>
                <p>{'detail' in item ? item.detail : ''}</p>
              </>
            )}
          />
          <SurfaceGrid
            title="Database surfaces"
            description="DB ve veri kaynakları ayrı operasyon katmanı olarak görünür."
            items={snapshot.databases}
            getBody={(item) => (
              <>
                <code>{'path' in item ? item.path : ''}</code>
                <p>{'detail' in item ? item.detail : ''}</p>
              </>
            )}
          />
          <SurfaceGrid
            title="Log surfaces"
            description="Bol loglu sistem için closure/release log yüzeyleri ayrıdır."
            items={snapshot.logs}
            getBody={(item) => (
              <>
                <code>{'path' in item ? item.path : ''}</code>
                <p>{'detail' in item ? item.detail : ''}</p>
              </>
            )}
          />
          <OperationsSection
            reports={snapshot.reports}
            commands={snapshot.commands}
            terminals={snapshot.terminals}
            blockers={snapshot.blockers}
            files={snapshot.files}
          />
        </main>
      </div>
      <Footer />
    </div>
  )
}

export default App
