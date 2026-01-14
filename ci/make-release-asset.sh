#!/usr/bin/env bash
# Builds the release and creates an archive
set -ex

BINARY_NAME=$1
OS=$2
TARGET=$3

if [[ -z "$GITHUB_REF" ]]
then
  echo "GITHUB_REF must be set"
  exit 1
fi

# Strip */tags/ from the start of the ref.
TAG=${GITHUB_REF#*/tags/}

host=$(rustc -Vv | grep ^host: | sed -e "s/host: //g")
export CARGO_PROFILE_RELEASE_LTO=true

cargo build --locked --bin $BINARY_NAME --release --target $TARGET

# Strip -unknown and -pc vendor identifiers from target for cleaner filenames
CLEAN_TARGET=${TARGET//-unknown/}
CLEAN_TARGET=${CLEAN_TARGET//-pc/}

cd target/$TARGET/release

case $OS in
  ubuntu*)
    asset="$BINARY_NAME-$TAG-$CLEAN_TARGET.tar.gz"
    tar czf ../../$asset $BINARY_NAME
    ;;
  macos*)
    asset="$BINARY_NAME-$TAG-$CLEAN_TARGET.tar.gz"
    tar czf ../../$asset $BINARY_NAME
    ;;
  windows*)
    asset="$BINARY_NAME-$TAG-$CLEAN_TARGET.zip"
    7z a ../../$asset $BINARY_NAME.exe
    ;;
  *)
    echo "OS should be second parameter, was: $OS"
    ;;
esac

cd ../..

if [[ -z "$GITHUB_ENV" ]]
then
  echo "GITHUB_ENV not set, run: gh release upload $TAG target/$asset"
else
  echo "ASSET_TAG=$TAG" >> $GITHUB_ENV
  echo "ASSET_PATH=target/$asset" >> $GITHUB_ENV
fi
