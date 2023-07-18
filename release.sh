curr_version=$(grep -m 1 "^version" Cargo.toml | awk -F '"' '{print $2}')

echo "Current version: $curr_version"
read -p "New version: " new_version

# replace the version in Cargo.toml
sed "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml > Cargo.toml.tmp
mv Cargo.toml.tmp Cargo.toml

# CHANGELOG.md
today=$(date +%Y-%m-%d)
unreleased_header="## [Unreleased]"
new_version_header="## [$new_version] - $today"
awk -v version="$new_version_header" -v unreleased="$unreleased_header" '
  /## \[Unreleased\]/ { print; print ""; print version; next }
  { print }
' CHANGELOG.md > CHANGELOG.md.tmp
mv CHANGELOG.md.tmp CHANGELOG.md

sleep 1

cargo update -p quartz-cli

git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Release v$new_version" && git tag -a "v$new_version" -m "Release v$new_version"

git push && git push --tags
