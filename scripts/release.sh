#!/bin/bash
set -euo pipefail

die() {
    echo "release: $*" >&2
    exit 1
}

version="${1:-}"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "usage: scripts/release.sh <major.minor.patch>"
tag="v$version"

cd "$(dirname "$0")/.."

[ "$(git rev-parse --abbrev-ref HEAD)" = "main" ] || die "releases are cut from main"
[ -z "$(git status --porcelain)" ] || die "working tree is not clean"
! git rev-parse -q --verify "refs/tags/$tag" >/dev/null || die "tag $tag already exists"

is_newer() {
    [ "$1" != "$2" ] && [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | tail -1)" = "$1" ]
}

current=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
is_newer "$version" "$current" || die "$version is not newer than Cargo.toml version $current"
latest_tag=$(git tag --list 'v*' --sort=-v:refname | head -1)
if [ -n "$latest_tag" ]; then
    is_newer "$version" "${latest_tag#v}" || die "$version is not newer than latest tag $latest_tag"
fi

sed -i.bak '1,/^version = /s/^version = "[^"]*"/version = "'"$version"'"/' Cargo.toml
rm Cargo.toml.bak
cargo check --quiet

git add Cargo.toml Cargo.lock
git commit -m "chore: release $tag"
git tag -a "$tag" -m "$tag"

echo
echo "Created commit and tag $tag. Check them, then push with:"
echo "  git push origin main $tag"
