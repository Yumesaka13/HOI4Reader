import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const cli = fileURLToPath(new URL('../frontend/node_modules/@tauri-apps/cli/tauri.js', import.meta.url));
const targets = process.argv.slice(2);
const result = spawnSync(process.execPath, [
  cli, 'android', 'build', '--debug', '--apk', '--split-per-abi', '--target',
  ...(targets.length ? targets : ['aarch64']),
], {
  stdio: 'inherit',
  env: {
    ...process.env,
    // Android's WebView can be older than the browsers targeted by Angular's default.
    BROWSERSLIST: 'Chrome 83',
    CARGO_PROFILE_DEV_OPT_LEVEL: '2',
    CARGO_PROFILE_DEV_DEBUG: '0',
  },
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
