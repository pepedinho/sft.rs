#!/usr/bin/env bash
set -euo pipefail

OUTDIR=generated_files
mkdir -p "$OUTDIR"

N=20
MIN=1024           # 1 KiB
MAX=$((80 * 1024 * 1024)) # 80 MiB

for i in $(seq 1 $N); do
  # produce a log-uniform random size
  # use awk to compute: size = int(10^(rand*log10(MAX/MIN)) * MIN)
  size=$(awk -v min=$MIN -v max=$MAX 'BEGIN {
    r = rand();
    # log-uniform between min and max
    s = exp(log(min) + r * (log(max) - log(min)));
    printf("%d", s);
  }')
  echo "creating $OUTDIR/file_${i}.bin of size $size bytes"
  # write in one shot from /dev/urandom
  head -c "$size" </dev/urandom > "$OUTDIR/file_${i}.bin"
done
