/**
 * Use Playwright to launch Obsidian and disable restricted mode so that
 * the Local REST API community plugin can be loaded automatically.
 */

import { _electron as electron } from 'playwright';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const obsidianExe = path.join(process.env.LOCALAPPDATA, 'Obsidian', 'Obsidian.exe');
const vaultPath = path.resolve(__dirname, '..', '..', 'obsidian-brain');

async function main() {
  console.log('Launching Obsidian...');
  const electronApp = await electron.launch({
    executablePath: obsidianExe,
    args: ['--vault', vaultPath],
  });

  const page = await electronApp.firstWindow();
  await page.waitForTimeout(5000);
  console.log('Opened window:', await page.title());

  // Open settings with Ctrl+,
  await page.keyboard.press('Control+,');
  await page.waitForTimeout(1000);

  // Click Community plugins in the left sidebar (text-based locator)
  const communityTab = page.locator('text=Community plugins');
  if (await communityTab.count() > 0) {
    await communityTab.first().click();
    await page.waitForTimeout(1000);
  } else {
    console.log('Could not find Community plugins tab; falling back.');
  }

  // Click "Turn off restricted mode" if present
  const turnOff = page.locator('button:has-text("Turn off restricted mode")');
  if (await turnOff.count() > 0) {
    await turnOff.first().click();
    await page.waitForTimeout(500);
    console.log('Clicked turn off restricted mode.');
  }

  // Confirm / reload if prompted
  const reload = page.locator('button:has-text("Turn on and reload")');
  if (await reload.count() > 0) {
    await reload.first().click();
    await page.waitForTimeout(5000);
    console.log('Confirmed reload.');
  }

  await page.waitForTimeout(3000);
  await electronApp.close();
  console.log('Done.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
