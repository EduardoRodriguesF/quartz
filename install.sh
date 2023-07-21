#!/bin/bash

get_latest_release() {
  curl --silent "https://api.github.com/repos/$1/releases/latest" | # Get latest release from GitHub api
    grep '"tag_name":' |                                            # Get tag line
    sed -E 's/.*"([^"]+)".*/\1/'                                    # Pluck JSON value
}


# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    platform="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    platform="macos"
else
    echo "Unsupported OS"
    exit 1
fi

# Detect architecture
arch=$(uname -m)

# Check if the platform and architecture are supported
if [ "$platform" == "linux" ]; then
    if [ "$arch" == "x86_64" ]; then
        target="x86_64-unknown-linux-musl"
    elif [ "$arch" == "armv7l" ]; then
        target="arm-unknown-linux-gnueabihf"
    else
        echo "Unsupported architecture for Linux"
        exit 1
    fi
elif [ "$platform" == "macos" ]; then
    if [ "$arch" == "x86_64" ]; then
        target="x86_64-apple-darwin"
    elif [ "$arch" == "arm64" ]; then
        target="aarch64-apple-darwin"
    else
        echo "Unsupported architecture for macOS"
        exit 1
    fi
fi

version=$(get_latest_release EduardoRodriguesF/quartz)
dir="quartz-$version-$target"

url="https://github.com/EduardoRodriguesF/quartz/releases/latest/download/$dir.tar.gz"

# Download and extract the tarball
echo "Downloading quartz $version for $platform ($target)..."
curl -L -o quartz.tar.gz $url
tar -xzf quartz.tar.gz

# Add the binary to a directory in $PATH
sudo mv $dir/quartz /usr/local/bin/
sudo cp $dir/doc/quartz.1 /usr/share/man/man1/

# Cleanup
rm quartz.tar.gz
rm -rf $dir

echo "quartz $version was installed"
