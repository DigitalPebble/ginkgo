#!/usr/bin/env bash
# Commits the badge JSON to a branch of the current repository, without
# touching the workspace: all git work happens in a temporary directory.
# Expects BADGE_PATH, BADGE_BRANCH and GITHUB_TOKEN in the environment.
set -euo pipefail

BADGE_PATH="${BADGE_PATH:-}"
BADGE_BRANCH="${BADGE_BRANCH:-badges}"

if [ -z "$BADGE_PATH" ] || [ ! -f "$BADGE_PATH" ]; then
  echo "::error::No badge file to publish (badge_path is empty or the file was not written)"
  exit 1
fi
if ! git check-ref-format --branch "$BADGE_BRANCH" >/dev/null 2>&1; then
  echo "::error::Invalid badge_branch: $BADGE_BRANCH"
  exit 1
fi

# The file keeps its name on the branch, whatever directory it was written to.
NAME="$(basename "$BADGE_PATH")"
WORK="$(mktemp -d "${RUNNER_TEMP:-/tmp}/aspen-badge.XXXXXX")"
cp "$BADGE_PATH" "$WORK.payload"
trap 'rm -rf "$WORK" "$WORK.payload"' EXIT

AUTH="$(printf 'x-access-token:%s' "$GITHUB_TOKEN" | base64 -w0)"
echo "::add-mask::$AUTH"

cd "$WORK"
git init --quiet
git remote add origin "${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY}.git"
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git config http.extraheader "Authorization: basic $AUTH"

for attempt in 1 2 3; do
  if git ls-remote --exit-code --heads origin "$BADGE_BRANCH" >/dev/null 2>&1; then
    git fetch --quiet --depth=1 origin "$BADGE_BRANCH"
    git checkout --quiet -f -B "$BADGE_BRANCH" FETCH_HEAD
  else
    git checkout --quiet --orphan "$BADGE_BRANCH"
  fi

  cp "$WORK.payload" "$NAME"
  git add "$NAME"
  if git diff --cached --quiet; then
    echo "Badge $NAME unchanged on $BADGE_BRANCH"
    exit 0
  fi
  git commit --quiet -m "Update carbon badge ($NAME)"

  if git push --quiet origin "$BADGE_BRANCH"; then
    echo "Published $NAME to branch $BADGE_BRANCH"
    exit 0
  fi
  echo "Push failed (attempt $attempt), retrying"
  sleep $((attempt * 2))
done

echo "::error::Could not push to $BADGE_BRANCH. The job needs 'permissions: contents: write'."
exit 1
