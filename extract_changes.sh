#!/bin/bash

# Usage: ./extract_changes.sh <version> <changelog_file>

# Check if the required arguments are provided
if [ $# -ne 2 ]; then
  echo "Usage: $0 <version> <changelog_file>"
  exit 1
fi

version=$1
version="${version#v}"  # Remove leading "v" if present
changelog_file=$2

# Find the starting and ending lines for the given version
start_line=$(grep -n -m 1 "## \[$version\]" "$changelog_file" | cut -d':' -f1)
end_line=$(grep -n "## \[" "$changelog_file" | grep -A1 "^$start_line:" | tail -1 | cut -d':' -f1)

# Extract the changes for the given version
changes=$(sed -n "$((start_line+1)),$((end_line-1))p" "$changelog_file")

# Output the changes
echo "$changes"
