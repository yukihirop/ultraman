#!/bin/bash

# Script to update Homebrew formula with new release
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.3.2"
    exit 1
fi

FORMULA_REPO="yukihirop/homebrew-tap"
FORMULA_FILE="Formula/ultraman.rb"

# Download release files to calculate SHA256
echo "Downloading release files to calculate SHA256..."
MAC_URL="https://github.com/yukihirop/ultraman/releases/download/v${VERSION}/ultraman-v${VERSION}-x86_64-mac.zip"
LINUX_URL="https://github.com/yukihirop/ultraman/releases/download/v${VERSION}/ultraman-v${VERSION}-x86_64-linux.zip"

MAC_SHA256=$(curl -sL "$MAC_URL" | shasum -a 256 | cut -d' ' -f1)
LINUX_SHA256=$(curl -sL "$LINUX_URL" | shasum -a 256 | cut -d' ' -f1)

echo "Mac SHA256: $MAC_SHA256"
echo "Linux SHA256: $LINUX_SHA256"

# Create updated formula content
cat > /tmp/ultraman.rb << EOF
# https://qiita.com/dalance/items/b07bee6cadfd4dd19756
class Ultraman < Formula
  version '${VERSION}'
  desc "Manage Procfile-based applications. (Rust Foreman)"
  homepage "https://github.com/yukihirop/ultraman"

  depends_on "rust" => :build
  
  if OS.mac?
    url "https://github.com/yukihirop/ultraman/releases/download/v${VERSION}/ultraman-v${VERSION}-x86_64-mac.zip"
    sha256 '${MAC_SHA256}'
  end

  if OS.linux?
    url "https://github.com/yukihirop/ultraman/releases/download/v${VERSION}/ultraman-v${VERSION}-x86_64-linux.zip"
    sha256 '${LINUX_SHA256}'
  end

  head 'https://github.com/yukihirop/ultraman.git'

  def install
    man1.install 'ultraman.1'
    bin.install 'ultraman'
  end
end
EOF

echo "Updated formula created at /tmp/ultraman.rb"
echo "Please manually update the formula in $FORMULA_REPO repository"