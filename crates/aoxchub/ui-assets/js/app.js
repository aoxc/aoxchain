console.log("navigation module loaded");

let state = null;
let selectedCommand = null;
let eventSource = null;

const ONBOARDING_KEY = 'aoxc.onboarding.v1';

function loadOnboarding() {
  try {
    return JSON.parse(localStorage.getItem(ONBOARDING_KEY) || '{}');
  } catch {
    return {};
  }
}

function saveOnboarding(next) {
  localStorage.setItem(ONBOARDING_KEY, JSON.stringify(next));
}

function abbreviateAddress(address) {
  if (!address || address.length < 12) return address || 'Not created';
  return `${address.slice(0, 8)}...${address.slice(-6)}`;
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
  document.querySelectorAll('.nav-item').forEach((node) => {
    node.classList.toggle('active', node.dataset.viewTarget === viewName);
  });
}

function renderHeader() {
  const onboarding = loadOnboarding();
  document.getElementById('header-environment').textContent = (state?.environment || 'mainnet').toUpperCase();
  document.getElementById('header-balance').textContent = onboarding.address ? `${onboarding.balance_placeholder || '0.00'} AOXC` : '-- AOXC';
  document.getElementById('header-address').textContent = abbreviateAddress(onboarding.address);
  document.getElementById('wallet-status').textContent = onboarding.address
    ? `Address ready: ${onboarding.address}`
    : 'No address found yet. Please create or import.';
}

function render() {
  if (!state) return;

  document.body.dataset.env = state.environment;
  document.getElementById('env-banner').textContent = state.banner;

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
  document.getElementById('binary-details').textContent = selected ? JSON.stringify(selected, null, 2) : 'No binary selected';

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
  const onboarding = loadOnboarding();
  if (onboarding.address) {
    alert('Address already exists. Use import only if you need to replace it.');
    return;
  }

  onboarding.address = generateLocalAddress();
  onboarding.balance_placeholder = '0.00';
  onboarding.created_at = new Date().toISOString();
  onboarding.source = 'generated_local';
  saveOnboarding(onboarding);
  renderHeader();
}

function importWallet() {
  const value = prompt('Enter existing AOXC address (public identifier only):', 'AOXC');
  if (!value) return;
  const trimmed = value.trim();
  if (!/^AOXC[a-fA-F0-9]{16,}$/.test(trimmed)) {
    alert('Invalid address format. Please provide a public AOXC address only.');
    return;
  }

  const onboarding = loadOnboarding();
  onboarding.address = trimmed;
  onboarding.balance_placeholder = onboarding.balance_placeholder || '0.00';
  onboarding.imported_at = new Date().toISOString();
  onboarding.source = 'imported_public_address';
  saveOnboarding(onboarding);
  renderHeader();
}

window.addEventListener('DOMContentLoaded', async () => {
  document.querySelectorAll('[data-view-target]').forEach((node) => {
    node.addEventListener('click', () => setView(node.dataset.viewTarget));
  });

  document.getElementById('go-wallet-from-landing').onclick = () => setView('wallet');
  document.getElementById('create-wallet').onclick = createWallet;
  document.getElementById('import-wallet').onclick = importWallet;

  document.getElementById('env-mainnet').onclick = () => setEnvironment('mainnet');
  document.getElementById('env-testnet').onclick = () => setEnvironment('testnet');

  document.getElementById('binary-select').onchange = async (e) => {
    await j('/api/binary/select', { method: 'POST', body: JSON.stringify({ binary_id: e.target.value }) });
    await refresh();
  };

  document.getElementById('add-custom').onclick = async () => {
    const path = document.getElementById('custom-binary').value.trim();
    if (!path) return;
    await j('/api/binary/custom', { method: 'POST', body: JSON.stringify({ path }) });
    document.getElementById('custom-binary').value = '';
    await refresh();
  };

  document.getElementById('execute').onclick = executeSelected;

  await refresh();
  renderHeader();
});
