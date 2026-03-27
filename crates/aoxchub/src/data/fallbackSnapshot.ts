// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

import type { ControlCenterSnapshot } from '../types/controlCenter'

export const fallbackSnapshot: ControlCenterSnapshot = {
  stage: 'desktop-control-bootstrap',
  verdict: 'needs-closure',
  overallPercent: 68,
  profile: 'aoxhub.desktop.admin',
  summary:
    'AOXHub admin panel artık zincirin kalbi olacak şekilde node, wallet, telemetry, explorer, terminal ve raporlama yüzeylerini aynı cockpit içinde toplamayı hedefliyor.',
  tracks: [
    {
      name: 'Mainnet readiness',
      percent: 60,
      summary: 'Mainnet profile, structured logs, genesis, node state ve operator key closure gerekiyor.',
      status: 'in-progress',
    },
    {
      name: 'Testnet parity',
      percent: 72,
      summary: 'Desktop orchestration deterministic testnet topology ile birebir hizalı olmalı.',
      status: 'in-progress',
    },
    {
      name: 'Desktop control center',
      percent: 75,
      summary: 'UI modüler ama gerçek command bridge, wallet ve terminal adaptörleri sonraki adım.',
      status: 'in-progress',
    },
  ],
  blockers: [
    {
      title: 'Three node orchestration',
      detail: 'Desktop panel local/test cluster yönetimini doğrudan CLI/Tauri komutlarına bağlamalı.',
      command: 'configs/deterministic-testnet/launch-testnet.sh',
    },
    {
      title: 'Wallet command bridge',
      detail: 'Wallet inspect/sign/export akışları GUI içinden komut adaptörleriyle yönetilmeli.',
      command: 'aoxc wallet inspect --profile mainnet',
    },
    {
      title: 'Unified reporting',
      detail: 'Launch, telemetry, audit ve forensics aynı export pipeline ile bağlanmalı.',
      command: 'scripts/validation/network_production_closure.sh --scenario soak',
    },
  ],
  files: [
    { label: 'Progress report', path: 'AOXC_PROGRESS_REPORT.md', exists: true },
    { label: 'Mainnet profile', path: 'configs/mainnet.toml', exists: true },
    { label: 'Testnet profile', path: 'configs/testnet.toml', exists: true },
  ],
  areas: [
    { name: 'Configuration', percent: 60, detail: 'Profile ve config drift kapanmalı.', status: 'in-progress' },
    { name: 'Network', percent: 100, detail: 'Peer/baseline kontrolleri hazır.', status: 'ready' },
    { name: 'Observability', percent: 50, detail: 'Structured logging ve telemetry export tamamlanmalı.', status: 'in-progress' },
    { name: 'Identity', percent: 0, detail: 'Genesis ve operator key yüzeyi eksik.', status: 'in-progress' },
  ],
  nodes: [
    {
      id: 'atlas',
      role: 'validator leader',
      status: 'online',
      chainId: 'AOXC-0077-MAIN',
      listenAddr: '127.0.0.1:39001',
      rpcAddr: '127.0.0.1:19101',
      peerCount: 4,
      securityMode: 'mutual_auth_test_fixture',
      command: 'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/atlas --rounds 12 --sleep-ms 200',
    },
    {
      id: 'boreal',
      role: 'validator follower',
      status: 'online',
      chainId: 'AOXC-0077-MAIN',
      listenAddr: '127.0.0.1:39002',
      rpcAddr: '127.0.0.1:19102',
      peerCount: 4,
      securityMode: 'mutual_auth_test_fixture',
      command: 'cargo run -q -p aoxcmd -- node-run --home configs/deterministic-testnet/homes/boreal --rounds 12 --sleep-ms 200',
    },
    {
      id: 'cypher',
      role: 'observer / telemetry anchor',
      status: 'degraded',
      chainId: 'AOXC-0077-MAIN',
      listenAddr: '127.0.0.1:39003',
      rpcAddr: '127.0.0.1:19103',
      peerCount: 4,
      securityMode: 'mutual_auth_test_fixture',
      command: 'cargo run -q -p aoxcmd -- node-health --home configs/deterministic-testnet/homes/cypher',
    },
  ],
  wallets: [
    {
      title: 'Operator wallet',
      route: 'mainnet guarded',
      status: 'connected',
      addressHint: 'AOXC1-VAL-OPER-PRIMARY',
      command: 'aoxc key-bootstrap --profile mainnet --password <value>',
      detail: 'Validator lifecycle, governance ve emergency operator eylemleri için ana yüzey.',
    },
    {
      title: 'Treasury wallet',
      route: 'dual-route mainnet/testnet',
      status: 'attention',
      addressHint: 'AOXC1-TREASURY-DESKTOP',
      command: 'aoxc wallet inspect --profile mainnet',
      detail: 'Transfer öncesi policy, audit ve route review gerektirir.',
    },
    {
      title: 'Recovery wallet',
      route: 'offline recovery lane',
      status: 'locked',
      addressHint: 'AOXC1-RECOVERY-ESCROW',
      command: 'aoxc diagnostics-bundle --redact',
      detail: 'Disaster recovery, cold storage ve rotation drill yüzeyi.',
    },
  ],
  telemetry: [
    {
      title: 'Mainnet RPC',
      status: 'ready',
      target: '0.0.0.0:8545',
      detail: 'Mainnet RPC hedefi desktop API/telemetry katmanı için görünür tutulur.',
    },
    {
      title: 'Testnet RPC',
      status: 'ready',
      target: '0.0.0.0:18545',
      detail: 'Testnet route parity ve desktop staging akışları için kullanılır.',
    },
    {
      title: 'Telemetry snapshot',
      status: 'blocked',
      target: 'artifacts/network-production-closure/telemetry-snapshot.json',
      detail: 'Prometheus ve alert evidence burada kapanmalı.',
    },
  ],
  reports: [
    {
      title: 'Release evidence bundle',
      status: 'ready',
      path: 'artifacts/release-evidence',
      detail: 'Compatibility matrix, SBOM ve provenance yüzeyi.',
    },
    {
      title: 'Network production closure',
      status: 'ready',
      path: 'artifacts/network-production-closure',
      detail: 'Soak, telemetry, recovery ve closure bundle burada toplanır.',
    },
    {
      title: 'Progress report',
      status: 'ready',
      path: 'AOXC_PROGRESS_REPORT.md',
      detail: 'Desktop cockpit bu readiness özetinden beslenir.',
    },
  ],
  commands: [
    {
      title: 'Bring up deterministic 3-node cluster',
      command: 'configs/deterministic-testnet/launch-testnet.sh',
      intent: 'Üç node local cluster bootstrap ve orchestration başlangıcı.',
    },
    {
      title: 'Generate production audit',
      command: 'cargo run -q -p aoxcmd -- production-audit --format json',
      intent: 'Operator denetim raporunu GUI reporting katmanına günceller.',
    },
    {
      title: 'Produce closure bundle',
      command: 'scripts/validation/network_production_closure.sh --scenario soak',
      intent: 'Telemetry, runtime ve rollout artefact setini üretir.',
    },
  ],
  databases: [
    {
      title: 'Runtime state store',
      status: 'ready',
      path: 'artifacts/',
      detail: 'Runtime state, release artifacts ve closure bundles için masaüstü veri yüzeyi.',
    },
    {
      title: 'Deterministic fixture store',
      status: 'ready',
      path: 'configs/deterministic-testnet/accounts.json',
      detail: 'Fixture account, node identity ve operator data seti.',
    },
  ],
  logs: [
    {
      title: 'Production closure logs',
      status: 'ready',
      path: 'artifacts/network-production-closure',
      detail: 'Soak, telemetry ve recovery log bundle yüzeyi.',
    },
    {
      title: 'Release evidence logs',
      status: 'ready',
      path: 'artifacts/release-evidence',
      detail: 'SBOM, provenance ve compatibility log/evidence yüzeyi.',
    },
  ],
  explorer: [
    {
      title: 'Progress explorer',
      status: 'ready',
      target: 'AOXC_PROGRESS_REPORT.md',
      detail: 'Readiness, blockers ve remediation order explorer.',
    },
    {
      title: 'Node fixture explorer',
      status: 'ready',
      target: 'configs/deterministic-testnet/nodes',
      detail: 'Node topology, RPC endpoint ve fixture explorer.',
    },
    {
      title: 'Artifact explorer',
      status: 'ready',
      target: 'artifacts/',
      detail: 'Release, audit, telemetry ve closure artefact explorer.',
    },
  ],
  terminals: [
    {
      title: 'Cluster terminal',
      command: 'configs/deterministic-testnet/launch-testnet.sh',
      detail: 'Deterministic local cluster başlatır.',
    },
    {
      title: 'Audit terminal',
      command: 'cargo run -q -p aoxcmd -- production-audit --format json',
      detail: 'Operator audit çıktısını üretir.',
    },
    {
      title: 'Closure terminal',
      command: 'scripts/validation/network_production_closure.sh --scenario soak',
      detail: 'Closure bundle ve telemetry output üretir.',
    },
  ],
}
