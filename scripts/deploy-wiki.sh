#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Forgeyard System Architecture Wiki Deployment Script
# Deploys the local wiki/ directory to GitHub Wiki repository (repo.wiki.git)
# ==============================================================================

REPO="${GITHUB_REPOSITORY:-irshadali5/forgeyard}"
WIKI_REPO_URL="https://github.com/${REPO}.wiki.git"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WIKI_SRC="${ROOT_DIR}/wiki"

echo "============================================================"
echo "Forgeyard Architecture Wiki Deployment"
echo "Target Repository: ${REPO}"
echo "Source Directory:  ${WIKI_SRC}"
echo "============================================================"

# Verify source directory exists
if [ ! -d "${WIKI_SRC}" ]; then
  echo "Error: Source wiki directory not found at ${WIKI_SRC}" >&2
  exit 1
fi

# Ensure wiki index and navigation files are up-to-date
echo "Regenerating wiki navigation index..."
python3 "${ROOT_DIR}/scripts/generate_wiki_index.py"

# Acquire GitHub token
TOKEN="${GITHUB_TOKEN:-}"
if [ -z "${TOKEN}" ]; then
  if command -v gh >/dev/null 2>&1; then
    echo "Retrieving authentication token from GitHub CLI (gh)..."
    TOKEN="$(gh auth token 2>/dev/null || true)"
  fi
fi

if [ -z "${TOKEN}" ]; then
  echo "Error: No GitHub token available. Set GITHUB_TOKEN or authenticate via 'gh auth login'." >&2
  exit 1
fi

AUTH_WIKI_URL="https://x-access-token:${TOKEN}@github.com/${REPO}.wiki.git"

# Prepare temporary deploy directory inside workspace / scratch
DEPLOY_DIR="${ROOT_DIR}/.wiki_deploy_tmp"
rm -rf "${DEPLOY_DIR}"
mkdir -p "${DEPLOY_DIR}"

trap 'rm -rf "${DEPLOY_DIR}"' EXIT

echo "Preparing wiki commit in staging directory..."
cd "${DEPLOY_DIR}"
git init -b master
git config user.name "${GIT_AUTHOR_NAME:-forgeyard-bot}"
git config user.email "${GIT_AUTHOR_EMAIL:-bot@forgeyard.dev}"

# Try cloning existing wiki if it exists
if git clone --depth 1 "${AUTH_WIKI_URL}" . 2>/dev/null; then
  echo "Successfully cloned existing GitHub Wiki repository."
  # Remove all tracked files except .git to cleanly mirror source
  find . -maxdepth 1 ! -name '.git' ! -name '.' -exec rm -rf {} +
else
  echo "Wiki remote repository is empty or not yet initialized."
fi

# Copy all files from wiki/
cp -r "${WIKI_SRC}/"* .

# Stage changes
git add -A

if git diff --staged --quiet; then
  echo "No changes detected between local wiki/ and remote GitHub Wiki. Everything is up to date."
  exit 0
fi

FILE_COUNT=$(find . -maxdepth 1 -name "*.md" | wc -l)
COMMIT_MSG="Sync system architecture wiki (${FILE_COUNT} specifications) [$(date -u +'%Y-%m-%d %H:%M:%S UTC')]"
git commit -m "${COMMIT_MSG}"

echo "Pushing ${FILE_COUNT} wiki documents to ${WIKI_REPO_URL} (branch: master)..."
if git push "${AUTH_WIKI_URL}" master:master --force; then
  echo "============================================================"
  echo "SUCCESS: GitHub Wiki deployed successfully!"
  echo "View Wiki at: https://github.com/${REPO}/wiki"
  echo "============================================================"
else
  echo "============================================================"
  echo "Notice: Push to GitHub Wiki failed. On GitHub, a repository's"
  echo "wiki Git backend is created when the first page is initialized."
  echo "If this repository has never had a wiki page saved in the web UI:"
  echo "1. Visit https://github.com/${REPO}/wiki"
  echo "2. Click 'Create the first page' and Save."
  echo "3. Re-run this script: ./scripts/deploy-wiki.sh"
  echo "============================================================"
  exit 1
fi
