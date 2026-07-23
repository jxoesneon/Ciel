/**
 * Use CDP to interact with Obsidian's UI and disable restricted mode.
 */

import { spawn } from 'node:child_process';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const obsidianExe = path.join(process.env.LOCALAPPDATA, 'Obsidian', 'Obsidian.exe');
const vaultPath = path.resolve(__dirname, '..', '..', 'obsidian-brain');

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

async function getDebuggerUrl() {
  for (let i = 0; i < 30; i++) {
    try {
      const res = await new Promise((resolve, reject) => {
        const req = http.get('http://127.0.0.1:9222/json', res => {
          let data = '';
          res.on('data', d => (data += d));
          res.on('end', () => resolve(data));
        });
        req.on('error', reject);
      });
      const pages = JSON.parse(res);
      if (pages.length > 0) return pages[0].webSocketDebuggerUrl;
    } catch {
      await sleep(500);
    }
  }
  throw new Error('Could not connect to CDP');
}

async function connectCdp(wsUrl) {
  const CDP = (await import('chrome-remote-interface')).default;
  return await CDP({ target: wsUrl });
}

async function findNodeByText(cdp, text) {
  const { root } = await cdp.DOM.getDocument();
  const res = await cdp.DOM.querySelectorAll({ nodeId: root.nodeId, selector: '*' });
  for (const nodeId of res.nodeIds) {
    try {
      const node = await cdp.DOM.describeNode({ nodeId, depth: 0 });
      if (node.node && node.node.nodeValue && node.node.nodeValue.includes(text)) {
        return node.node.nodeId;
      }
    } catch {}
  }
  return null;
}

async function clickByText(cdp, text) {
  const nodeId = await findNodeByText(cdp, text);
  if (!nodeId) throw new Error(`Could not find element with text: ${text}`);
  const box = await cdp.DOM.getBoxModel({ nodeId });
  const { content } = box.model;
  const x = (content[0] + content[2]) / 2;
  const y = (content[1] + content[5]) / 2;
  await cdp.Input.dispatchMouseEvent({ type: 'mousePressed', x, y, button: 'left', clickCount: 1 });
  await cdp.Input.dispatchMouseEvent({ type: 'mouseReleased', x, y, button: 'left', clickCount: 1 });
}

async function pressKeyCombo(cdp, keys) {
  for (const key of keys) {
    await cdp.Input.dispatchKeyEvent({ type: 'keyDown', key });
  }
  for (const key of [...keys].reverse()) {
    await cdp.Input.dispatchKeyEvent({ type: 'keyUp', key });
  }
}

async function main() {
  console.log('Launching Obsidian with remote debugging...');
  const child = spawn(obsidianExe, ['--vault', vaultPath, '--remote-debugging-port=9222'], {
    detached: true,
    stdio: 'ignore',
  });
  child.unref();

  await sleep(10000);
  console.log('Connecting to CDP...');
  const wsUrl = await getDebuggerUrl();
  const cdp = await connectCdp(wsUrl);

  console.log('Opening settings...');
  await pressKeyCombo(cdp, ['Control', ',']);
  await sleep(2000);

  console.log('Clicking Community plugins...');
  try {
    await clickByText(cdp, 'Community plugins');
  } catch (e) {
    console.log('Could not click Community plugins by text:', e.message);
  }
  await sleep(2000);

  console.log('Clicking Turn off restricted mode...');
  try {
    await clickByText(cdp, 'Turn off restricted mode');
  } catch (e) {
    console.log('Could not click Turn off restricted mode:', e.message);
  }
  await sleep(2000);

  console.log('Clicking Turn on and reload if present...');
  try {
    await clickByText(cdp, 'Turn on and reload');
  } catch (e) {
    console.log('No reload confirmation:', e.message);
  }
  await sleep(5000);

  await cdp.close();
  console.log('Done.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
