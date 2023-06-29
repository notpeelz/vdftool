#!/usr/bin/env bash

set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"

# XXX: Docker doesn't like "." as a volume path
PROJECT_DIR="$(realpath "${PROJECT_DIR:-}")"

if [[ ! -d "${PROJECT_DIR:-}" ]]; then
  echo "Invalid PROJECT_DIR"
  exit 1
fi

if [[ -z "${BUILD_TARGETS:-}" ]]; then
  echo "Invalid BUILD_TARGETS"
  exit 1
fi

echo "Building docker image"
docker_image="$(docker build -q -f "$SCRIPT_DIR/Dockerfile" "$SCRIPT_DIR")"

echo "Building project for targets: ${BUILD_TARGETS//$'\n'/, }"
readarray -t artifacts <<< "$(
  docker run \
    --rm \
    --volume "$PROJECT_DIR":/root/src \
    -e BUILD_TARGETS="$BUILD_TARGETS" \
    --workdir /root/src \
    "$docker_image" \
    bash -c '
      args=()

      readarray -t targets <<< "$BUILD_TARGETS"
      for target in "${targets[@]}"; do
        [[ -z "$target" ]] && continue
        args+=("--target=$target")
      done

      cargo build --message-format json --release "${args[@]}"
    ' \
    | jq -rs '.[] | select(.reason == "compiler-artifact") | .executable | values' \
    | tee /dev/stdout
)"

for i in "${!artifacts[@]}"; do
  artifacts["$i"]="$(realpath -sm --relative-to=/root/src "${artifacts["$i"]}")"
done

json='{}'
for artifact in "${artifacts[@]}"; do
  [[ -z "$artifact" ]] && continue
  target="$(sed -E 's:^target/([^/]*)/.*$:\1:' <<< "$artifact")"
  json="$(
    jq -c \
      --arg key "$target" \
      --arg value "$artifact" \
      '. += {($key): $value}' \
    <<< "$json"
  )"
done

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "artifacts=$json" >> "$GITHUB_OUTPUT"
else
  echo "$json"
fi
