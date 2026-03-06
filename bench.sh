#!/bin/bash
# Benchmark lngterm vs pyterm
set -e
cd "$(dirname "$0")"
BINDIR="$(pwd)/target/release"
PYTERM="$(pwd)/pyterm/pyterm.py"
SOCAT_PID=""

cleanup() {
    [ -n "$SOCAT_PID" ] && kill "$SOCAT_PID" 2>/dev/null || true
    rm -f /tmp/vbench0 /tmp/vbench1 /tmp/tm.txt /tmp/lng_startup.txt /tmp/py_startup.txt
}
trap cleanup EXIT

DEV=/tmp/vbench0
BAUD=115200

echo "=== Startup time (5 runs each) ==="
echo ""

echo "[1/3] Creating virtual serial pair with socat..."
echo "socat pty,raw,echo=0,link=$DEV pty,raw,echo=0,link=/tmp/vbench1"
socat -d -d pty,raw,echo=0,link="$DEV" pty,raw,echo=0,link=/tmp/vbench1 2>/dev/null &
SOCAT_PID=$!
sleep 0.5

if [ ! -e "$DEV" ]; then
    echo "ERROR: socat did not create $DEV"
    exit 1
fi

echo ""
echo "[2/3] Measuring lngterm startup..."
echo "lngterm:"
for i in 1 2 3 4 5; do
    /usr/bin/time -f "%e" -o /tmp/tm.txt sh -c "'$BINDIR/lngterm' -d $DEV -b $BAUD 2>&1 | head -1" >/dev/null 2>&1 || true
    cat /tmp/tm.txt
done | tee /tmp/lng_startup.txt

echo ""
echo "[3/3] Measuring pyterm startup..."
echo "pyterm:"
for i in 1 2 3 4 5; do
    /usr/bin/time -f "%e" -o /tmp/tm.txt sh -c "python3 '$PYTERM' -d $DEV -b $BAUD 2>&1 | head -1" >/dev/null 2>&1 || true
    cat /tmp/tm.txt
done | tee /tmp/py_startup.txt

echo ""
echo "=== Summary (avg of 5 runs) ==="
echo "See above per-run times for lngterm and pyterm; you can compute the average startup time if needed."
