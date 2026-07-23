/**
 * Try to enable the Obsidian Local REST API plugin via Obsidian's internal API.
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

async function evalCdp(cdp, expression) {
  const result = await cdp.Runtime.evaluate({ expression, awaitPromise: true });
  return result.result.value;
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

  console.log('Checking app availability...');
  const hasApp = await evalCdp(cdp, 'typeof window.app !== "undefined"');
  console.log('window.app available:', hasApp);

  if (hasApp) {
    console.log('Trying to enable plugin...');
    try {
      const result = await evalCdp(cdp, `
        (async () => {
          try {
            if (window.app.plugins) {
              await window.app.plugins.loadManifests();
              window.app.plugins.setEnable('obsidian-local-rest-api', true);
              await window.app.plugins.enablePlugin('obsidian-local-rest-api');
              return 'plugin enabled';
            }
            return 'no plugins object';
          } catch (e) {
            return e.message;
          }
        })()
      `);
      console.log('Result:', result);
    } catch (e) {
      console.log('Error:', e.message);
    }
  }

  await cdp.close();
  console.log('Done.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
