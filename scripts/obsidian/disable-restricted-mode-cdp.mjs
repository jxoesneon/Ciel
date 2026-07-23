/**
 * Use the Chrome DevTools Protocol to toggle off Obsidian's restricted mode
 * by setting the internal localStorage flag and restarting Obsidian.
 */

import { spawn } from 'node:child_process';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const obsidianExe = path.join(process.env.LOCALAPPDATA, 'Obsidian', 'Obsidian.exe');
const vaultPath = path.resolve(__dirname, '..', '..', 'obsidian-brain');
const appIdPath = path.join(process.env.APPDATA, 'Obsidian', 'id');

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

async function getAppId() {
  return (await fs.readFile(appIdPath, 'utf8')).trim();
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

async function main() {
  const appId = await getAppId();
  const key = `enable-plugin-${appId}`;
  console.log(`AppId: ${appId}, key: ${key}`);

  console.log('Launching Obsidian with remote debugging...');
  const child = spawn(obsidianExe, ['--vault', vaultPath, '--remote-debugging-port=9222'], {
    detached: true,
    stdio: 'ignore',
  });
  child.unref();

  await sleep(8000);
  console.log('Connecting to CDP...');
  const wsUrl = await getDebuggerUrl();
  const cdp = await connectCdp(wsUrl);

  console.log('Setting localStorage flag...');
  await cdp.Runtime.evaluate({
    expression: `localStorage.setItem('${key}', 'true'); 'done'`,
  });

  await cdp.close();
  console.log('Closing Obsidian for restart...');
  try {
    child.kill();
  } catch {}
  await sleep(3000);

  console.log('Restarting Obsidian...');
  const child2 = spawn(obsidianExe, ['--vault', vaultPath], {
    detached: true,
    stdio: 'ignore',
  });
  child2.unref();

  await sleep(8000);
  console.log('Done.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
