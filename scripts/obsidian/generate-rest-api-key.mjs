/**
 * Generate a fresh API key and self-signed certificate for the Obsidian Local REST API
 * plugin and write them to the plugin's data.json. This lets Ciel authenticate without
 * the user having to copy the key from Obsidian's settings UI.
 */

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import forge from 'node-forge';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const pluginDir = path.resolve(__dirname, '..', '..', 'obsidian-brain', '.obsidian', 'plugins', 'obsidian-local-rest-api');
const dataPath = path.join(pluginDir, 'data.json');

function generateApiKey() {
  const md = forge.md.sha256.create();
  md.update(forge.random.getBytesSync(128));
  return md.digest().toHex();
}

function generateCertificate() {
  const expiry = new Date();
  const today = new Date();
  expiry.setDate(today.getDate() + 365);

  const keypair = forge.pki.rsa.generateKeyPair(2048);
  const attrs = [{ name: 'commonName', value: 'Obsidian Local REST API' }];

  const certificate = forge.pki.createCertificate();
  certificate.setIssuer(attrs);
  certificate.setSubject(attrs);

  const subjectAltNames = [{ type: 7, ip: '127.0.0.1' }];

  certificate.setExtensions([
    { name: 'basicConstraints', cA: true, critical: true },
    {
      name: 'keyUsage',
      keyCertSign: true,
      digitalSignature: true,
      nonRepudiation: true,
      keyEncipherment: false,
      dataEncipherment: false,
      critical: true,
    },
    {
      name: 'extKeyUsage',
      serverAuth: true,
      clientAuth: true,
      codeSigning: true,
      emailProtection: true,
      timeStamping: true,
    },
    {
      name: 'nsCertType',
      client: true,
      server: true,
      email: true,
      objsign: true,
      sslCA: true,
      emailCA: true,
      objCA: true,
    },
    { name: 'subjectAltName', altNames: subjectAltNames },
  ]);

  certificate.serialNumber = '1';
  certificate.publicKey = keypair.publicKey;
  certificate.validity.notAfter = expiry;
  certificate.validity.notBefore = today;
  certificate.sign(keypair.privateKey, forge.md.sha256.create());

  return {
    cert: forge.pki.certificateToPem(certificate),
    privateKey: forge.pki.privateKeyToPem(keypair.privateKey),
    publicKey: forge.pki.publicKeyToPem(keypair.publicKey),
  };
}

async function main() {
  await fs.mkdir(pluginDir, { recursive: true });

  const apiKey = generateApiKey();
  const crypto = generateCertificate();

  const data = {
    apiKey,
    crypto,
    port: 27124,
    insecurePort: 27123,
    enableInsecureServer: true,
    enableVerboseLogging: false,
    bindingHost: '127.0.0.1',
  };

  await fs.writeFile(dataPath, JSON.stringify(data, null, 2), 'utf8');
  console.log(`Wrote ${dataPath}`);
  console.log(`API key: ${apiKey}`);
  console.log('The insecure REST API is enabled on http://127.0.0.1:27123');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
