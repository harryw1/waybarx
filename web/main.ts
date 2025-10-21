// Minimal demo that shows modern web UI running in a layer-shell bar.
// Dev server: Vite at http://127.0.0.1:5173 (configure in src/main.rs)

declare global { interface Window { __nativeReceive?: (msg:any)=>void; webkit?: any } }

const workspaces = document.querySelector('#workspaces') as HTMLDivElement;
const clock = document.querySelector('#clock') as HTMLSpanElement;
const systemInfo = document.querySelector('#system-info') as HTMLSpanElement;

function renderWorkspaces(names: string[]) {
  workspaces.innerHTML = names.map((n, i) =>
    `<span class="ws" ${i===0 ? 'data-active' : ''}>${n}</span>`).join('');
}

function tick() {
  const now = new Date();
  clock.textContent = now.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'});
}
setInterval(tick, 1000); tick();

// Handle messages from Rust
window.__nativeReceive = (msg) => {
  if (msg.ok && msg.cmd === 'workspaces' && msg.data) {
    renderWorkspaces(msg.data);
  } else if (msg.ok && msg.cmd === 'system_info' && msg.data) {
    renderSystemInfo(msg.data);
  } else if (!msg.ok) {
    console.error('Error from Rust:', msg.error || msg);
  } else if (msg.echo) {
    console.log('Rust echoed:', msg.echo);
  }
};

function renderSystemInfo(data: any) {
  const cpu = Math.round(data.cpu.usage_percent);
  const mem = Math.round(data.memory.usage_percent);
  const memUsed = (data.memory.used_bytes / (1024**3)).toFixed(1);
  const memTotal = (data.memory.total_bytes / (1024**3)).toFixed(1);

  systemInfo.innerHTML = `CPU: ${cpu}% | MEM: ${mem}% (${memUsed}/${memTotal}GB)`;
}

function postNative(payload: any) {
  // WebKitGTK message channel named "native"
  window.webkit?.messageHandlers?.native?.postMessage?.(payload);
}

// Request workspace data on load
function fetchWorkspaces() {
  postNative({ cmd: 'get_workspaces' });
}

// Initial fetch and periodic updates (every 2 seconds)
// TODO: Re-enable after fixing Hyprland IPC connection issues
// The hyprland crate panics when the socket isn't found, which happens when:
// - Not running in a proper Hyprland session
// - Environment variables (HYPRLAND_INSTANCE_SIGNATURE) aren't set correctly
// fetchWorkspaces();
// setInterval(fetchWorkspaces, 2000);

// Fetch system info
function fetchSystemInfo() {
  postNative({ cmd: 'get_system_info' });
}

// Fetch system info every 2 seconds
fetchSystemInfo();
setInterval(fetchSystemInfo, 2000);
