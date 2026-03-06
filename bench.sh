#!/bin/bash
# Benchmark lngterm vs pyterm
set -e
cd "$(dirname "$0")"
BINDIR="$(pwd)/target/release"
PYTERM="$(pwd)/pyterm/pyterm.py"
SOCAT_PID=""

cleanup() {
    [ -n "$SOCAT_PID" ] && kill $SOCAT_PID 2>/dev/null || true
    rm -f /tmp/vbench0 /tmp/vbench1
}
trap cleanup EXIT

# Create virtual serial pair
socat -d -d pty,raw,echo=0,link=/tmp/vbench0 pty,raw,echo=0,link=/tmp/vbench1 2>/dev/null &
SOCAT_PID=$!
sleep 0.5
[ ! -e /tmp/vbench0 ] && { echo "socat failed"; exit 1; }

DEV=/tmp/vbench0
BAUD=115200

echo "=== Startup time (5 runs each) ==="
echo ""
echo "lngterm:"
for i in 1 2 3 4 5; do
    /usr/bin/time -f "%e" -o /tmp/tm.txt sh -c "'$BINDIR/lngterm' -d $DEV -b $BAUD 2>&1 | head -1"
    cat /tmp/tm.txt
done | tee /tmp/lng_startup.txt

echo ""
echo "pyterm:"
for i in 1 2 3 4 5; do
    /usr/bin/time -f "%e" -o /tmp/tm.txt sh -c "python3 '$PYTERM' -d $DEV -b $BAUD 2>&1 | head -1"
    cat /tmp/tm.txt
done | tee /tmp/py_startup.txt

echo ""
echo "=== Memory (RSS, idle) ==="
kill $SOCAT_PID 2>/dev/null; wait $SOCAT_PID 2>/dev/null || true
rm -f /tmp/vbench0 /tmp/vbench1
sleep 0.3
socat -d -d pty,raw,echo=0,link=/tmp/vbench0 pty,raw,echo=0,link=/tmp/vbench1 2>/dev/null &
SOCAT_PID=$!
sleep 0.8

echo -n "lngterm: "
script -q -c "'$BINDIR/lngterm' -d /tmp/vbench0 -b $BAUD" /dev/null 2>/dev/null &
SPID=$!
sleep 0.5
LPID=$(pgrep -P $SPID 2>/dev/null | head -1)
LRSS=$(grep VmRSS /proc/$LPID/status 2>/dev/null | awk '{print $2}' || echo "0")
kill -9 $SPID $LPID 2>/dev/null; wait 2>/dev/null || true
echo "${LRSS} kB"

sleep 0.2
echo -n "pyterm: "
script -q -c "python3 '$PYTERM' -d /tmp/vbench0 -b $BAUD" /dev/null 2>/dev/null &
SPID2=$!
sleep 0.5
PYID=$(pgrep -P $SPID2 2>/dev/null | head -1)
PRSS=$(grep VmRSS /proc/$PYID/status 2>/dev/null | awk '{print $2}' || echo "0")
kill -9 $SPID2 $PYID 2>/dev/null; wait 2>/dev/null || true
echo "${PRSS} kB"

echo ""
echo "=== Summary (for README) ==="
echo "lngterm startup (avg): $(grep -E '^[0-9.]+$' /tmp/lng_startup.txt 2>/dev/null | awk '{s+=$1;n++} END{printf "%.3f", n>0?s/n:"N/A"}') s"
echo "pyterm startup (avg):  $(grep -E '^[0-9.]+$' /tmp/py_startup.txt 2>/dev/null | awk '{s+=$1;n++} END{printf "%.3f", n>0?s/n:"N/A"}') s"
echo "lngterm RSS: $LRSS kB"
echo "pyterm RSS: $PRSS kB"
