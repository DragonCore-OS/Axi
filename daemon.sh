#!/bin/bash
cd /home/admin/axi_pure
echo "⚡ AXI Genesis Node starting..."
./target/release/axi genesis 2>/dev/null || true
while true; do
    echo "[$(date)] AXI Heartbeat | Status: $(./target/release/axi status 2>&1)"
    sleep 60
done
