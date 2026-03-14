#!/bin/bash
# AXI 纯血版系统部署脚本

echo "╔════════════════════════════════════════════════════════════╗"
echo "║              ⚡ AXI 纯血版 - 系统部署                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

if [ "$EUID" -ne 0 ]; then 
    echo "❌ 请使用 sudo 运行此脚本"
    echo "   sudo bash ~/axi_pure/install.sh"
    exit 1
fi

# 1. 创建systemd服务
echo "🔧 创建systemd服务..."

cat > /etc/systemd/system/axi-genesis.service << 'EOF'
[Unit]
Description=AXI Genesis Node - Physical-anchored currency
After=network.target

[Service]
Type=simple
User=admin
WorkingDirectory=/home/admin/axi_pure
ExecStart=/home/admin/axi_pure/target/release/axi status
Restart=always
RestartSec=30
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# 2. 重新加载systemd
systemctl daemon-reload

# 3. 启用服务
systemctl enable axi-genesis

# 4. 启动服务
systemctl start axi-genesis

echo ""
echo "✅ 部署完成!"
echo ""
echo "服务状态:"
systemctl status axi-genesis --no-pager

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 管理命令:"
echo "  sudo systemctl start axi-genesis    # 启动"
echo "  sudo systemctl stop axi-genesis     # 停止"
echo "  sudo systemctl restart axi-genesis  # 重启"
echo "  sudo systemctl status axi-genesis   # 查看状态"
echo "  sudo journalctl -u axi-genesis -f   # 查看日志"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
