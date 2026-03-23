import type { NodeControl, WalletSurface } from '../../types/controlCenter'

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

export function NodeWalletSection({ nodes, wallets }: { nodes: NodeControl[]; wallets: WalletSurface[] }) {
  return (
    <section className="section-block split-layout split-layout-wide">
      <div>
        <div className="section-heading">
          <h2>3-node control plane</h2>
          <p>Validator, follower, observer ve RPC yüzeyleri ayrı kartlarda yönetilir.</p>
        </div>
        <div className="stack-list">
          {nodes.map((node) => (
            <article className="info-card compact panel-surface" key={node.id}>
              <div className="card-topline">
                <div>
                  <h3>{node.id}</h3>
                  <p className="muted">{node.role}</p>
                </div>
                <span className={`status-pill ${node.status}`}>{statusLabel(node.status)}</span>
              </div>
              <dl className="detail-grid">
                <div><dt>Chain</dt><dd>{node.chainId}</dd></div>
                <div><dt>Listen</dt><dd>{node.listenAddr}</dd></div>
                <div><dt>RPC</dt><dd>{node.rpcAddr}</dd></div>
                <div><dt>Peers</dt><dd>{node.peerCount}</dd></div>
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
          <h2>Wallet lanes</h2>
          <p>Walletler ayrık ve görev odaklı şekilde yönetilir.</p>
        </div>
        <div className="stack-list">
          {wallets.map((wallet) => (
            <article className="info-card compact panel-surface" key={wallet.title}>
              <div className="card-topline">
                <h3>{wallet.title}</h3>
                <span className={`status-pill ${wallet.status}`}>{statusLabel(wallet.status)}</span>
              </div>
              <p>{wallet.detail}</p>
              <dl className="detail-grid">
                <div><dt>Route</dt><dd>{wallet.route}</dd></div>
                <div><dt>Address</dt><dd>{wallet.addressHint}</dd></div>
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
  )
}
