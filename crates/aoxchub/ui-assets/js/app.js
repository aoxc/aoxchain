let state = null;
let selectedCommand = null;
let eventSource = null;

const UI_STATE_KEY = 'aoxc.ui-state.v2';

function loadUiState() {
  try {
    return JSON.parse(localStorage.getItem(UI_STATE_KEY) || '{}');
  } catch {
    return {};
  }
}

function saveUiState(next) {
  localStorage.setItem(UI_STATE_KEY, JSON.stringify(next));
}

function abbreviateAddress(address) {
  if (!address || address.length < 16) return address || 'No address yet';
  return `${address.slice(0, 10)}...${address.slice(-8)}`;
}

function generateLocalAddress() {
  const bytes = new Uint8Array(20);
  crypto.getRandomValues(bytes);
  return `AOXC${Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')}`;
}

async function j(url, options = {}) {
  const res = await fetch(url, { headers: { 'content-type': 'application/json' }, ...options });
  if (!res.ok) {
    const t = await res.text();
    throw new Error(t || `HTTP ${res.status}`);
  }
  return res.json();
}

function setView(viewName) {
  document.querySelectorAll('.view').forEach((node) => {
    node.classList.toggle('active', node.dataset.view === viewName);
  });
  document.querySelectorAll('.nav-item[data-view-target]').forEach((node) => {
    node.classList.toggle('active', node.dataset.viewTarget === viewName);
  });

  const ui = loadUiState();
  ui.last_view = viewName;
  saveUiState(ui);
}

function upsertWallet(patch) {
  const ui = loadUiState();
  ui.wallet = { ...(ui.wallet || {}), ...patch };
  ui.onboarding_completed = Boolean(ui.wallet.address);
  saveUiState(ui);
}

function walletData() {
  return loadUiState().wallet || {};
}

function terminalFooter(message) {
  const node = document.getElementById('terminal-footer-note');
  if (node) node.textContent = message;
}

function appendTerminalOutput(line) {
  const targets = [document.getElementById('terminal'), document.getElementById('terminal-output')];
  targets.filter(Boolean).forEach((node) => {
    node.textContent += `${line}\n`;
    node.scrollTop = node.scrollHeight;
  });
}

function clearTerminalOutput() {
  const targets = [document.getElementById('terminal'), document.getElementById('terminal-output')];
  targets.filter(Boolean).forEach((node) => {
    node.textContent = '';
  });
}


function listToHtml(list) {
  return (list || []).map((item) => `<li>${item}</li>`).join('');
}

function renderDashboard() {
  const dashboard = state?.dashboard;
  if (!dashboard) return;

  const metrics = [
    ['Chain', dashboard.chain_name],
    ['Network', `${dashboard.network_kind} (${dashboard.network_id})`],
    ['Height', `${dashboard.current_height}`],
    ['Finalized', `${dashboard.finalized_height}`],
    ['Round', `${dashboard.current_round}`],
    ['Validators', `${dashboard.validator_count}`],
    ['Observers', `${dashboard.observer_count}`],
    ['Peers', `${dashboard.connected_peers}`],
    ['Node', dashboard.local_node_status],
    ['RPC', dashboard.rpc_status],
    ['P2P', dashboard.p2p_status],
    ['Health', dashboard.health_status],
    ['Genesis FP', dashboard.genesis_fingerprint],
  ];

  const metricsNode = document.getElementById('dashboard-metrics');
  metricsNode.innerHTML = metrics
    .map(([k, v]) => `<article class="dashboard-metric"><span>${k}</span><strong>${v}</strong></article>`)
    .join('');

  const versions = dashboard.installed_versions || {};
  const versionsNode = document.getElementById('dashboard-versions');
  versionsNode.innerHTML = [
    ['aoxc', versions.aoxc || 'unknown'],
    ['aoxchub', versions.aoxchub || 'unknown'],
    ['runtime', versions.runtime || 'unknown'],
  ]
    .map(([k, v]) => `<article class="dashboard-metric"><span>${k}</span><strong>${v}</strong></article>`)
    .join('');

  document.getElementById('dashboard-events').innerHTML = listToHtml(dashboard.last_events);
  document.getElementById('dashboard-txs').innerHTML = listToHtml(dashboard.last_txs);
  document.getElementById('dashboard-warnings').innerHTML = listToHtml(dashboard.last_warnings);
  document.getElementById('dashboard-actions').innerHTML = (dashboard.quick_actions || [])
    .map((action) => `<span class="dashboard-chip">${action}</span>`)
    .join('');
}

function renderHeader() {
  const wallet = walletData();
  const env = (state?.environment || 'mainnet').toUpperCase();
  const walletReady = Boolean(wallet.address);

  document.getElementById('header-environment').textContent = env;
  document.getElementById('header-wallet-state').textContent = walletReady ? 'Ready' : 'Not ready';
  document.getElementById('header-balance').textContent = walletReady
    ? `${wallet.balance_placeholder || '0.00'} AOXC`
    : '-- AOXC';
  document.getElementById('header-address').textContent = walletReady
    ? abbreviateAddress(wallet.address)
    : 'No address yet';

  const readinessNode = document.getElementById('wallet-readiness');
  const statusNode = document.getElementById('wallet-status');
  readinessNode.textContent = walletReady ? 'Wallet status: ready' : 'Wallet status: not ready';
  statusNode.textContent = walletReady
    ? `${wallet.label ? `${wallet.label} • ` : ''}${wallet.address}`
    : 'No address found yet. Create a new one or import an existing public address.';

  const terminalWallet = document.getElementById('terminal-wallet-status');
  if (terminalWallet) {
    terminalWallet.textContent = walletReady
      ? `Wallet status: ready (${abbreviateAddress(wallet.address)})`
      : 'Wallet status: not ready';
  }

  document.getElementById('wallet-label').value = wallet.label || '';
}

function updateSelectionViews(command) {
  selectedCommand = command;
  document.getElementById('selected-label').textContent = command?.spec?.label || '';
  document.getElementById('preview').textContent = command?.preview || '';
  document.getElementById('execute').disabled = !command?.allowed;

  const terminalLabel = document.getElementById('terminal-selected-label');
  const terminalPreview = document.getElementById('terminal-preview');
  const terminalRunSelected = document.getElementById('terminal-run-selected');
  const terminalMeta = document.getElementById('terminal-selected-meta');
  if (terminalLabel) terminalLabel.textContent = command?.spec?.label || '';
  if (terminalPreview) terminalPreview.textContent = command?.preview || '';
  if (terminalRunSelected) terminalRunSelected.disabled = !command?.allowed;
  if (terminalMeta) {
    terminalMeta.textContent = command
      ? `${command.spec.group} • ${command.spec.risk} • ${command.policy_note}`
      : 'No command selected.';
  }
}

function renderCommandCards(listNode, className = 'card') {
  if (!listNode) return;
  listNode.innerHTML = '';
  state.commands.forEach((c) => {
    const card = document.createElement('button');
    card.className = className;
    card.disabled = !c.allowed;
    card.innerHTML = `<strong>${c.spec.label}</strong><small>${c.spec.group} • ${c.spec.risk}</small><small>${c.spec.description}</small><small>${c.policy_note}</small>`;
    card.onclick = () => updateSelectionViews(c);
    listNode.appendChild(card);
  });
}

function render() {
  if (!state) return;

  const isMainnet = state.environment === 'mainnet';
  document.body.dataset.env = state.environment;
  document.body.dataset.theme = state.environment;
  document.getElementById('env-banner').textContent = state.banner;
  document.getElementById('home-environment-banner').textContent = state.banner;
  const terminalEnv = document.getElementById('terminal-env-banner');
  if (terminalEnv) terminalEnv.textContent = state.banner;
  document.getElementById('env-mainnet').classList.toggle('active', isMainnet);
  document.getElementById('env-testnet').classList.toggle('active', !isMainnet);
  document.getElementById('header-env-message').textContent = isMainnet
    ? 'Mainnet: production guardrails and certified binaries only.'
    : 'Testnet: exploratory mode with custom binary support for safe experimentation.';
  document.getElementById('hero-title').textContent = isMainnet
    ? 'Operate AOXChain with production-grade discipline.'
    : 'Explore AOXChain behavior safely in test mode.';
  document.getElementById('hero-env-caption').textContent = isMainnet
    ? 'Mainnet actions carry production risk. Review previews before execution.'
    : 'Testnet mode emphasizes experimentation. Validate command outcomes before promoting to Mainnet.';
  document.getElementById('wallet-env-note').textContent = isMainnet
    ? 'Mainnet mode is active. Use a production-safe address and review every action carefully.'
    : 'Testnet mode is active. Use disposable addresses and iterate workflows without production impact.';

  const binSelect = document.getElementById('binary-select');
  binSelect.innerHTML = '';
  state.binaries.forEach((b) => {
    const o = document.createElement('option');
    o.value = b.id;
    o.textContent = `${b.kind} | ${b.version || 'unknown'} | ${b.trust}`;
    if (state.selected_binary_id === b.id) o.selected = true;
    binSelect.appendChild(o);
  });

  const selected = state.binaries.find((b) => b.id === state.selected_binary_id);
  const selectedBinaryText = selected ? JSON.stringify(selected, null, 2) : 'No binary selected';
  document.getElementById('binary-details').textContent = selectedBinaryText;
  const terminalBinary = document.getElementById('terminal-binary-details');
  if (terminalBinary) terminalBinary.textContent = selectedBinaryText;

  renderCommandCards(document.getElementById('commands'));
  renderCommandCards(document.getElementById('terminal-commands'), 'card terminal-card');

  if (selectedCommand) {
    const refreshed = state.commands.find((c) => c.spec.id === selectedCommand.spec.id);
    updateSelectionViews(refreshed || null);
  }

  renderHeader();
  renderDashboard();
}

async function refresh() {
  state = await j('/api/state');
  render();
}

async function setEnvironment(environment) {
  await j('/api/environment', { method: 'POST', body: JSON.stringify({ environment }) });
  const ui = loadUiState();
  ui.selected_environment = environment;
  saveUiState(ui);
  await refresh();
}

async function waitForJob(jobId) {
  while (true) {
    const job = await j(`/api/jobs/${jobId}`);
    if (job.finished_at) return job;
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
}

async function executeCommand(command, options = {}) {
  if (!command) return;
  if (eventSource) eventSource.close();

  const out = await j('/api/execute', {
    method: 'POST',
    body: JSON.stringify({ command_id: command.spec.id, confirm: true }),
  });

  if (options.clear) clearTerminalOutput();

  eventSource = new EventSource(`/api/jobs/${out.job_id}/stream`);
  eventSource.onmessage = (e) => {
    appendTerminalOutput(e.data);
  };

  terminalFooter(`Running ${command.spec.label}...`);
  const job = await waitForJob(out.job_id);
  eventSource.close();
  terminalFooter(`Finished ${command.spec.label} with exit code ${job.exit_code}.`);
}

async function executeSelected() {
  if (!selectedCommand) return;
  const ok = window.confirm(`Execute command?\n\n${selectedCommand.preview}`);
  if (!ok) return;
  await executeCommand(selectedCommand, { clear: true });
}

async function executeAutonomousRun() {
  const allowed = state.commands.filter((c) => c.allowed);
  if (!allowed.length) {
    alert('No allowed command is available in the active environment.');
    return;
  }

  const ok = window.confirm(
    `Autonomous mode will execute ${allowed.length} allowed commands in sequence. Continue?`,
  );
  if (!ok) return;

  clearTerminalOutput();
  for (let i = 0; i < allowed.length; i += 1) {
    const cmd = allowed[i];
    updateSelectionViews(cmd);
    appendTerminalOutput(`[autonomous ${i + 1}/${allowed.length}] ${cmd.spec.label}`);
    // eslint-disable-next-line no-await-in-loop
    await executeCommand(cmd, { clear: false });
  }
  terminalFooter(`Autonomous sequence completed (${allowed.length} commands).`);
}

function createWallet() {
  const wallet = walletData();
  if (wallet.address) {
    alert('An active address already exists. Import only if you intend to replace it.');
    return;
  }

  const label = document.getElementById('wallet-label').value.trim();
  upsertWallet({
    address: generateLocalAddress(),
    label,
    balance_placeholder: '0.00',
    created_at: new Date().toISOString(),
    source: 'generated_local',
  });
  renderHeader();
  renderDashboard();
}

function importWallet(event) {
  event.preventDefault();
  const address = document.getElementById('wallet-address-input').value.trim();
  if (!/^AOXC[a-fA-F0-9]{16,}$/.test(address)) {
    alert('Please provide a valid public AOXC address. Private key material is never requested here.');
    return;
  }

  const label = document.getElementById('wallet-label').value.trim();
  upsertWallet({
    address,
    label,
    balance_placeholder: '0.00',
    imported_at: new Date().toISOString(),
    source: 'imported_public_address',
  });
  document.getElementById('wallet-address-input').value = '';
  renderHeader();
  renderDashboard();
}

async function applyStoredEnvironmentPreference() {
  const ui = loadUiState();
  if (ui.selected_environment && ui.selected_environment !== state.environment) {
    await setEnvironment(ui.selected_environment);
  }
}

window.addEventListener('DOMContentLoaded', async () => {
  document.querySelectorAll('[data-view-target]').forEach((node) => {
    node.addEventListener('click', () => setView(node.dataset.viewTarget));
  });

  document.getElementById('go-wallet-from-landing').onclick = () => setView('wallet');
  document.getElementById('create-wallet').onclick = createWallet;
  document.getElementById('import-wallet-form').addEventListener('submit', importWallet);

  document.getElementById('env-mainnet').onclick = () => setEnvironment('mainnet');
  document.getElementById('env-testnet').onclick = () => setEnvironment('testnet');

  document.getElementById('binary-select').onchange = async (e) => {
    await j('/api/binary/select', { method: 'POST', body: JSON.stringify({ binary_id: e.target.value }) });
    await refresh();
  };

  document.getElementById('add-custom').onclick = async () => {
    if (state?.environment === 'mainnet') {
      alert('Mainnet mode accepts only certified release binaries.');
      return;
    }
    const path = document.getElementById('custom-binary').value.trim();
    if (!path) return;
    await j('/api/binary/custom', { method: 'POST', body: JSON.stringify({ path }) });
    document.getElementById('custom-binary').value = '';
    await refresh();
  };

  document.getElementById('execute').onclick = executeSelected;
  document.getElementById('terminal-run-selected').onclick = executeSelected;
  document.getElementById('terminal-run-autonomous').onclick = executeAutonomousRun;
  document.getElementById('terminal-clear').onclick = () => {
    clearTerminalOutput();
    terminalFooter('Terminal output cleared.');
  };

  await refresh();
  await applyStoredEnvironmentPreference();

  const lastView = loadUiState().last_view;
  if (lastView === 'wallet' || lastView === 'home' || lastView === 'dashboard' || lastView === 'terminal') {
    setView(lastView);
  }
  renderHeader();
  renderDashboard();
});
