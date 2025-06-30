# 部署说明文档

## Valor-go Team

## 1. LLM部署相关的性能指标列表

1. 首Token返回延迟(Time to First Token)
   1. 定义：从用户发送请求到收到第一个输出Token的时间
   2. 合理性：直接影响用户体验的“响应感知”速度，对于交互场景十分重要
2. 输出吞吐量（输出速度, Tokens per Second）
   1. 定义：模型每秒生成的Token数量（排除首Token延迟时间）
   2. 合理性：决定长文本生成的流畅度。低吞吐量会导致长回答“卡顿式输出”，影响用户体验
3. 并发处理能力(Concurrent Requests Handled)
   1. 定义：系统在保证SLA（Service Level Agreement，如延迟<2秒）下能同时处理的请求数
   2. 合理性：反映系统的实际服务容量，直接影响部署成本和业务扩展性（如应对流量高峰）
4. 显存利用率(GPU Memory Utilization)
   1. 定义：推理时GPU显存占用与总显存的百分比
   2. 合理性：过高的显存占用会导致OOM错误，限制并发能力；过低则可能意味着资源浪费，需优化模型分片或量化策略
5. 错误率(Error Rate)
   1. 定义：请求失败比例（如超时、崩溃、返回非法内容）
   2. 合理性：直接影响服务可靠性；需区分暂时性错误（可重试）和系统性错误（如显存泄漏）
6. 每请求成本(Cost per Request)
   1. 定义：单次请求消耗的计算资源成本（如GPU秒数）
   2. 合理性：直接关联商业可行性，尤其在面向C端的高频调用场景（如AI写作助手）

## 2. 编译相关配置（强烈推荐和文档配置保持一致）

1. 操作系统：Ubuntu 22.04
2. Driver Version: 550.163.01
3. CUDA Version: 12.4
4. gcc (Ubuntu 11.4.0-1ubuntu1~22.04) 11.4.0
5. g++ (Ubuntu 11.4.0-1ubuntu1~22.04) 11.4.0

说明：

1. Driver Version，CUDA Version，gcc，g++各版本需要保持一致，否则可能无法编译
2. 不建议使用clang，CUDA对clang支持不太好

## 3. 获取模型

1. 使用：Qwen3-1.7B
2. 指令：由于国内防火墙的问题，我们使用镜像来实现
   1. `export HF_ENDPOINT=https://hf-mirror.com`配置hugging-face镜像访问
   2. cd 到 `.../lab4/llama.cpp/build/bin`然后使用指令 `./llama-server -hf ggml-org/Qwen3-1.7B-GGUF: Q4_K_M`即可获取模型并且打开端口进行对话
   3. "mv ~/.cache/llama.cpp/ggml-org_Qwen3-1.7B-GGUF_Qwen3-1.7B-Q4_K_M.gguf .../lab4/llama.cpp/build/bin/models/"将模型存储到buid/bin文件夹中方便使用
