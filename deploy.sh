#!/bin/bash
# AXI 纯血版部署脚本

echo "⚡ AXI Pure 部署"

# 1. 停止旧服务
pkill -f axi_server 2>/dev/null
pkill -f "python3.*axi" 2>/dev/null
echo "✓ 旧服务已停止"

# 2. 安装到系统
cd ~/axi_pure
cargo build --release 2>&1 | tail -5

# 3. 创建systemd服务
sudo tee /etc/systemd/system/axi-genesis.service > /dev/null << 'EOF'
[Unit]
Description=AXI Genesis Node - Physical-anchored currency
After=network.target

[Service]
Type=simple
User=admin
WorkingDirectory=/home/admin/axi_pure
ExecStart=/home/admin/axi_pure/target/release/axi
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable axi-genesis

echo ""
echo "✓ 部署完成"
echo ""
echo "启动命令:"
echo "  sudo systemctl start axi-genesis"
echo "  sudo systemctl status axi-genesis"
