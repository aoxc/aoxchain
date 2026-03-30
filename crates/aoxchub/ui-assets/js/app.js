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

  document.getElementById('wallet-label').value = wallet.label || '';
}

function render() {
  if (!state) return;

  const isMainnet = state.environment === 'mainnet';
  document.body.dataset.env = state.environment;
  document.body.dataset.theme = state.environment;
  document.getElementById('env-banner').textContent = state.banner;
  document.getElementById('home-environment-banner').textContent = state.banner;
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
  document.getElementById('env-binding-slug').textContent = state.binding.slug;
  document.getElementById('env-binding-config').textContent = state.binding.root_config;
  document.getElementById('env-binding-home').textContent = state.binding.aoxc_home;
  document.getElementById('env-binding-make').textContent = state.binding.make_scope;

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
  document.getElementById('binary-details').textContent = selected
    ? JSON.stringify(selected, null, 2)
    : 'No binary selected';

  const commands = document.getElementById('commands');
  commands.innerHTML = '';
  state.commands.forEach((c) => {
    const card = document.createElement('button');
    card.className = 'card';
    card.disabled = !c.allowed;
    card.innerHTML = `<strong>${c.spec.label}</strong><small>${c.spec.group} • ${c.spec.risk}</small><small>${c.spec.description}</small><small>${c.policy_note}</small>`;
    card.onclick = () => {
      selectedCommand = c;
      document.getElementById('selected-label').textContent = c.spec.label;
      document.getElementById('preview').textContent = c.preview;
      document.getElementById('execute').disabled = !c.allowed;
    };
    commands.appendChild(card);
  });

  renderHeader();
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

async function executeSelected() {
  if (!selectedCommand) return;
  const ok = window.confirm(`Execute command?\n\n${selectedCommand.preview}`);
  if (!ok) return;

  const out = await j('/api/execute', {
    method: 'POST',
    body: JSON.stringify({ command_id: selectedCommand.spec.id, confirm: true }),
  });

  if (eventSource) eventSource.close();
  eventSource = new EventSource(`/api/jobs/${out.job_id}/stream`);
  const terminal = document.getElementById('terminal');
  terminal.textContent = '';
  eventSource.onmessage = (e) => {
    terminal.textContent += `${e.data}\n`;
    terminal.scrollTop = terminal.scrollHeight;
  };
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

  await refresh();
  await applyStoredEnvironmentPreference();

  const lastView = loadUiState().last_view;
  if (lastView === 'wallet' || lastView === 'home') {
    setView(lastView);
  }
  renderHeader();
});
