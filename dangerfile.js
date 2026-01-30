// Danger.js configuration for automated PR reviews
const { danger, warn, fail, message } = require('danger');

// Check PR size
const bigPRThreshold = 500;
const additions = danger.github.pr.additions;
const deletions = danger.github.pr.deletions;
const totalChanges = additions + deletions;

if (totalChanges > bigPRThreshold) {
  warn(`This PR is quite large (${totalChanges} lines changed). Consider breaking it into smaller PRs for easier review.`);
}

// Check for WIP
const title = danger.github.pr.title;
if (title.includes('WIP') || title.includes('wip') || title.includes('[WIP]')) {
  warn('This PR is marked as Work In Progress.');
}

// Check description
const description = danger.github.pr.body;
if (!description || description.length < 10) {
  warn('Please provide a meaningful PR description.');
}

// Check for test changes when src changes
const srcChanges = danger.git.modified_files.filter(f => f.startsWith('src/'));
const testChanges = danger.git.modified_files.filter(f => f.includes('test') || f.startsWith('tests/'));

if (srcChanges.length > 0 && testChanges.length === 0) {
  warn('This PR modifies source code but has no test changes. Consider adding tests.');
}

// Check for Cargo.toml changes
const cargoChanged = danger.git.modified_files.includes('Cargo.toml');
const lockChanged = danger.git.modified_files.includes('Cargo.lock');

if (cargoChanged && !lockChanged) {
  warn('Cargo.toml was modified but Cargo.lock was not updated. Run `cargo build` to update the lockfile.');
}

// Celebrate small PRs
if (totalChanges < 50) {
  message('🎉 Nice small PR! Easy to review.');
}

// Check for AGENTS.md updates when workflow changes
const workflowChanges = danger.git.modified_files.filter(f => f.includes('.github/workflows/'));
const agentsMdChanged = danger.git.modified_files.includes('AGENTS.md');

if (workflowChanges.length > 0 && !agentsMdChanged) {
  message('CI workflow changed - consider updating AGENTS.md if commands changed.');
}
