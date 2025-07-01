#!/bin/bash

# 默认参数值
N_GEN=128
N_PROMPT=512
BATCH_SIZE=2048
THREADS=16
N_GPU_LAYERS=99
N_DEPTH=0

# 帮助信息
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -n <number>      Number of tokens to generate (default: 128)"
    echo "  -p <number>      Prompt length (default: 512)"
    echo "  -b <number>      Batch size (default: 2048)"
    echo "  -t <number>      Number of threads (default: system dependent)"
    echo "  -ngl <number>    Number of GPU layers (default: 99)"
    echo "  -d <number>      Depth/context length (default: 0)"
    echo "  -h, --help       Show this help message"
    echo ""
    echo "Example:"
    echo "  $0 -n 256 -p 1024 -b 512"
    echo ""
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -n)
            N_GEN="$2"
            shift 2
            ;;
        -p)
            N_PROMPT="$2"
            shift 2
            ;;
        -b)
            BATCH_SIZE="$2"
            shift 2
            ;;
        -t)
            THREADS="$2"
            shift 2
            ;;
        -ngl)
            N_GPU_LAYERS="$2"
            shift 2
            ;;
        -d)
            N_DEPTH="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# 构建GPU日志文件名
GPU_LOG_FILE="gpu_log_${N_GEN}_${N_PROMPT}_${BATCH_SIZE}_${THREADS}_${N_GPU_LAYERS}_${N_DEPTH}.csv"

# 显示当前配置
echo "=========================================="
echo "Benchmark Configuration:"
echo "  -n (tokens to generate): $N_GEN"
echo "  -p (prompt length): $N_PROMPT"
echo "  -b (batch size): $BATCH_SIZE"
echo "  -t (threads): $THREADS"
echo "  -ngl (GPU layers): $N_GPU_LAYERS"
echo "  -d (depth): $N_DEPTH"
echo "GPU log file: $GPU_LOG_FILE"
echo "=========================================="

# 检查必要文件是否存在
if [ ! -f "./llama-bench" ]; then
    echo "Error: llama-bench not found in current directory"
    exit 1
fi

if [ ! -f "./models/ggml-org_Qwen3-1.7B-GGUF_Qwen3-1.7B-Q4_K_M.gguf" ]; then
    echo "Error: Model file not found at ./models/ggml-org_Qwen3-1.7B-GGUF_Qwen3-1.7B-Q4_K_M.gguf"
    exit 1
fi

# 检查nvidia-smi是否可用
if ! command -v nvidia-smi &> /dev/null; then
    echo "Error: nvidia-smi not found"
    exit 1
fi

# 构建llama-bench命令
LLAMA_CMD="./llama-bench -n $N_GEN -p $N_PROMPT -b $BATCH_SIZE -ngl $N_GPU_LAYERS -d $N_DEPTH -m ./models/ggml-org_Qwen3-1.7B-GGUF_Qwen3-1.7B-Q4_K_M.gguf"

# 如果threads不是默认值，添加-t参数
if [ "$THREADS" != 16 ]; then
    LLAMA_CMD="$LLAMA_CMD -t $THREADS"
fi

echo "Starting GPU monitoring..."
# 启动GPU监控（后台运行）
nvidia-smi --query-gpu=timestamp,utilization.gpu,utilization.memory,memory.total,memory.used --format=csv -l 1 > "$GPU_LOG_FILE" &
GPU_MONITOR_PID=$!

# 等待一秒确保GPU监控开始
sleep 1

echo "Starting benchmark..."
echo "Command: $LLAMA_CMD"
echo ""

# 运行llama-bench
$LLAMA_CMD

# 获取llama-bench的退出状态
BENCH_EXIT_CODE=$?

# 停止GPU监控
echo ""
echo "Stopping GPU monitoring..."
kill $GPU_MONITOR_PID 2>/dev/null

# 等待GPU监控进程完全结束
wait $GPU_MONITOR_PID 2>/dev/null

echo "=========================================="
echo "Benchmark completed!"
echo "GPU log saved to: $GPU_LOG_FILE"

# 显示GPU日志文件的行数（排除标题行）
if [ -f "$GPU_LOG_FILE" ]; then
    LOG_LINES=$(wc -l < "$GPU_LOG_FILE")
    echo "GPU log contains $LOG_LINES lines of data"
    echo ""
    echo "Preview of GPU log (first 5 lines):"
    head -5 "$GPU_LOG_FILE"
fi

echo "=========================================="

exit $BENCH_EXIT_CODE
