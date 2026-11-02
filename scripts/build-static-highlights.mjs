import { build } from '../apps/web/node_modules/esbuild/lib/main.js';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
const output = process.argv[2];
if (!output) throw Error('Usage: node scripts/build-static-highlights.mjs OUTPUT');
await build({ entryPoints: ['scripts/static-highlights-entry.mjs'], outfile: output, bundle: true, platform: 'node', format: 'esm', target: 'node22', minify: true,
  plugins: [{ name: 'embedded-parser', setup(build) {
    build.onResolve({ filter: /\.wasm\?url$/ }, args => ({ path: path.resolve(args.resolveDir, args.path.replace(/\?url$/, '')), namespace: 'embedded-wasm' }));
    build.onLoad({ filter: /.*/, namespace: 'embedded-wasm' }, async args => ({ contents: 'export default '+JSON.stringify('data:application/wasm;base64,'+(await readFile(args.path)).toString('base64')), loader: 'js' }));
  } }],
});
