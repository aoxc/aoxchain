let state = null;
let selectedCommand = null;
let eventSource = null;

async function j(url, options = {}) {
  const res = await fetch(url, { headers: { 'content-type': 'application/json' }, ...options });
  if (!res.ok) {
    const t = await res.text();
    throw new Error(t || `HTTP ${res.status}`);
  }
  return res.json();
}

function render() {
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
}

async function refresh() { state = await j('/api/state'); render(); }

async function setEnvironment(environment) {
  await j('/api/environment', { method: 'POST', body: JSON.stringify({ environment }) });
  await refresh();
}

async function executeSelected() {
  if (!selectedCommand) return;
  const ok = window.confirm(`Execute command?\n\n${selectedCommand.preview}`);
  if (!ok) return;
  const out = await j('/api/execute', { method: 'POST', body: JSON.stringify({ command_id: selectedCommand.spec.id, confirm: true }) });
  if (eventSource) eventSource.close();
  eventSource = new EventSource(`/api/jobs/${out.job_id}/stream`);
  const terminal = document.getElementById('terminal');
  terminal.textContent = '';
  eventSource.onmessage = (e) => {
    terminal.textContent += e.data + '\n';
    terminal.scrollTop = terminal.scrollHeight;
  };
}

window.addEventListener('DOMContentLoaded', async () => {
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
});
