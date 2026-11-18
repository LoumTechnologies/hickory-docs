import * as Y from '../apps/web/node_modules/yjs/dist/yjs.mjs';
import * as encoding from '../apps/web/node_modules/lib0/encoding.js';
import * as decoding from '../apps/web/node_modules/lib0/decoding.js';
import * as sync from '../apps/web/node_modules/y-protocols/sync.js';
import WebSocket from '../apps/web/node_modules/ws/wrapper.mjs';
const [origin, identifier, expected] = process.argv.slice(2);
const doc = new Y.Doc();
const socket = new WebSocket(origin.replace(/^http/, 'ws') + '/api/ws?doc=' + encodeURIComponent('doc:' + identifier));
const deadline = setTimeout(() => { console.error('Owned room did not synchronize'); process.exit(1); }, 15000);
let ready = false;
socket.on('open', () => {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, 0);
  encoding.writeVarUint(encoder, 0);
  sync.writeSyncStep1(encoder, doc);
  socket.send(encoding.toUint8Array(encoder));
});
socket.on('message', (bytes) => {
  const frame = new Uint8Array(bytes);
  if (frame[0] !== 0) return;
  const decoder = decoding.createDecoder(frame.subarray(1));
  if (decoding.readVarUint(decoder) !== 0) return;
  const type = decoding.readVarUint(decoder);
  // Deliberately do not answer server step1: even a no-op client step2 would
  // mark the production room mutated and schedule its debounce persistence.
  if (type !== sync.messageYjsSyncStep2) return;
  Y.applyUpdate(doc, decoding.readVarUint8Array(decoder), socket);
  const source = doc.getText('source').toString();
  if (source !== expected) throw new Error('Foreign room bytes differ from fixture');
  if (!ready) {
    ready = true;
    clearTimeout(deadline);
    process.stdout.write(JSON.stringify({ready: true, source, writes_sent: 0}) + '\n');
  }
});
socket.on('error', (error) => { console.error(error); process.exitCode = 1; });
process.stdin.on('data', () => socket.close());
socket.on('close', () => { clearTimeout(deadline); doc.destroy(); process.exit(ready ? 0 : 1); });
