#!/bin/bash
# 实验状态检查脚本 - 明早运行查看结果

echo "=== AXI Phase A 实验状态报告 ==="
echo "生成时间: $(date)"
echo ""

EXPERIMENT_DIR="/home/admin/DragonCore-OS-Axi/experiments"
LOG_DIR="$EXPERIMENT_DIR/logs"
RESULTS_DIR="$EXPERIMENT_DIR/results"

echo "[进程状态]"
if [ -f "$EXPERIMENT_DIR/ralph_orchestrator.pid" ]; then
    PID=$(cat "$EXPERIMENT_DIR/ralph_orchestrator.pid")
    if ps -p $PID > /dev/null 2>&1; then
        echo "✅ Ralph Orchestrator 运行中 (PID: $PID)"
        echo "运行时间: $(ps -o etime= -p $PID)"
    else
        echo "⏹️ Ralph Orchestrator 已停止"
    fi
else
    echo "❌ 未找到进程文件"
fi

echo ""
echo "[日志文件]"
ls -lh $LOG_DIR/*.log 2>/dev/null | tail -5

echo ""
echo "[结果文件]"
echo "已生成的小时报告:"
ls -1 $RESULTS_DIR/hour_*_metrics.json 2>/dev/null | wc -l | xargs echo "  - Metrics:"
ls -1 $RESULTS_DIR/hour_*_decision.json 2>/dev/null | wc -l | xargs echo "  - Decisions:"

echo ""
echo "[最新小时结果]"
LATEST_METRICS=$(ls -t $RESULTS_DIR/hour_*_metrics.json 2>/dev/null | head -1)
if [ -n "$LATEST_METRICS" ]; then
    echo "文件: $(basename $LATEST_METRICS)"
    cat $LATEST_METRICS | python3 -m json.tool 2>/dev/null || cat $LATEST_METRICS
fi

echo ""
echo "[最终报告]"
if [ -f "$RESULTS_DIR/FINAL_REPORT.json" ]; then
    echo "✅ 实验已完成，最终报告已生成"
    cat "$RESULTS_DIR/FINAL_REPORT.json" | python3 -m json.tool 2>/dev/null | head -30
else
    echo "⏳ 实验进行中，最终报告尚未生成"
fi

echo ""
echo "=== 查看实时日志 ==="
echo "tail -f $LOG_DIR/ralph_$(date +%Y%m%d).log"
