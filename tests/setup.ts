import { execFileSync } from 'child_process';

/** Rebuild better-sqlite3 for the system Node.js (tests don't run inside Electron). */
export function setup() {
  execFileSync('npm', ['rebuild', 'better-sqlite3'], { stdio: 'pipe' });
}

/** Rebuild better-sqlite3 back for Electron so the app still works after testing. */
export function teardown() {
  try {
    execFileSync('node_modules/.bin/electron-rebuild', ['-f', '-w', 'better-sqlite3'], { stdio: 'pipe' });
  } catch {
    // Non-critical — developer can run `npm run postinstall` manually if needed
  }
}
