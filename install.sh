#!/usr/bin/env bash
set -e

REPO="WideBandAI/gan-cv-simulator"

VERSION=$(curl -s https://api.github.com/repos/$REPO/releases/latest \
  | grep tag_name \
  | cut -d '"' -f4)

ARCH=$(uname -m)
OS=$(uname -s)

if [ "$OS" = "Linux" ]; then
  TARGET="x86_64-unknown-linux-musl"
elif [ "$OS" = "Darwin" ]; then
  TARGET="x86_64-apple-darwin"
else
  echo "Unsupported OS"
  exit 1
fi

URL="https://github.com/$REPO/releases/download/$VERSION/gan-cv-simulator-$VERSION-$TARGET.tar.gz"

curl -L $URL -o gan-cv-simulator.tar.gz
tar xf gan-cv-simulator.tar.gz

chmod +x gan-cv-simulator
sudo mv gan-cv-simulator /usr/local/bin/

echo "Installed gan-cv-simulator $VERSION"