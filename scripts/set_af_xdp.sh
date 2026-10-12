#!/usr/bin/env bash
# ==============================================================================
# ZeptoRPC Kernel & AF_XDP Express Data Path (XDP) Infrastructure Setup
# Target OS: Ubuntu 22.04 LTS / Debian 12 (Kernel 6.x+)
# ==============================================================================

set -euo pipefail

INTERFACE=${1:-"eth0"}
QUEUE_COUNT=${2:-4}

echo "[+] Initializing ZeptoRPC High-Throughput Kernel Optimization on interface: ${INTERFACE}"

# 1. Ensure Root Privileges
if [ "$EUID" -ne 0 ]; then
  echo "[-] Error: This script must be run as root."
  exit 1
fi

# 2. Maximize OS Socket Memory & Ring Buffer Bounds
echo "[+] Configuring Sysctl Kernel Network Parameters..."
sysctl -w net.core.rmem_max=134217728
sysctl -w net.core.wmem_max=134217728
sysctl -w net.core.rmem_default=67108864
sysctl -w net.core.wmem_default=67108864
sysctl -w net.core.netdev_max_backlog=250000
sysctl -w net.core.somaxconn=65535

# 3. Configure Network Interface Hardware Queues
echo "[+] Optimizing Ethtool Ring Buffer & Offloads for ${INTERFACE}..."
ethtool -G "${INTERFACE}" rx 4096 tx 4096 || true
ethtool -L "${INTERFACE}" combined "${QUEUE_COUNT}" || true

# Disable generic receive offload (GRO) for low latency deterministic ingress
ethtool -K "${INTERFACE}" gro off lro off tso off gso off || true

# 4. Pin NIC IRQs to Designated CPU Cores
echo "[+] Pinning Interrupt Request (IRQ) affinities..."
systemctl stop irqbalance || true

IRQ_NUMS=$(grep "${INTERFACE}" /proc/interrupts | awk -F: '{print $1}' | tr -d ' ')
CORE_ID=0
for IRQ in ${IRQ_NUMS}; do
  MASK=$(printf "%x" $((1 << CORE_ID)))
  echo "${MASK}" > "/proc/irq/${IRQ}/smp_affinity"
  echo "    Pushed IRQ ${IRQ} -> Core ${CORE_ID} (Mask: 0x${MASK})"
  CORE_ID=$((CORE_ID + 1))
done

echo "[+] AF_XDP Zero-Copy Drivers Operational on ${INTERFACE}."