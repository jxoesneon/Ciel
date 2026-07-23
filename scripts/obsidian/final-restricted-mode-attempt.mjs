/**
 * Final attempt: use Obsidian's internal app API via CDP to open settings and disable restricted mode.
 */

import http from 'node:http';
import CDP from 'chrome-remote-interface';

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

async function getTargets() {
  return new Promise((resolve, reject) => {
    http.get('http://127.0.0.1:9222/json', res => {
      let data = '';
      res.on('data', d => (data += d));
      res.on('end', () => resolve(JSON.parse(data)));
    }).on('error', reject);
  });
}

async function evalExpr(client, expression) {
  const result = await client.Runtime.evaluate({ expression, awaitPromise: true, returnByValue: true });
  return result.result?.value;
}

async function main() {
  const targets = await getTargets();
  const page = targets.find(t => t.type === 'page');
  if (!page) throw new Error('No page target found');

  const client = await CDP({ target: page.webSocketDebuggerUrl });

  console.log('Checking window.app...');
  const hasApp = await evalExpr(client, 'typeof window.app !== "undefined"');
  console.log('window.app available:', hasApp);

  if (!hasApp) {
    console.log('window.app not available. Cannot proceed with internal API automation.');
    await client.close();
    return;
  }

  console.log('Opening settings via command...');
  const openResult = await evalExpr(client, `
    (async () => {
      try {
        window.app.commands.executeCommandById('app:open-settings');
        return 'opened';
      } catch (e) { return e.message; }
    })()
  `);
  console.log('Open settings result:', openResult);
  await sleep(1000);

  console.log('Attempting to find and click restricted mode toggle...');
  const clickResult = await evalExpr(client, `
    (() => {
      const buttons = Array.from(document.querySelectorAll('button, .setting-item-control'));
      const toggle = buttons.find(b => b.textContent && b.textContent.includes('Turn off')) ||
                     buttons.find(b => b.textContent && b.textContent.includes('restricted')) ||
                     document.querySelector('[aria-label*="Restricted"]') ||
                     document.querySelector('input[type="checkbox"]');
      if (toggle) {
        toggle.click();
        return 'clicked: ' + (toggle.textContent || toggle.ariaLabel || toggle.type);
      }
      return 'no restricted toggle found';
    })()
  `);
  console.log('Click result:', clickResult);

  await client.close();
}

main().catch(err => {
  console.error(err.message);
  process.exit(1);
});
