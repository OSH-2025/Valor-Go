# 性能测试与分析文档

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
4. 显存占用(GPU Memory Use)
   1. 定义：推理时GPU显存占用
   2. 合理性：过高的显存占用会导致OOM错误，限制并发能力；过低则可能意味着资源浪费，需优化模型分片或量化策略
5. 错误率(Error Rate)
   1. 定义：请求失败比例（如超时、崩溃、返回非法内容）
   2. 合理性：直接影响服务可靠性；需区分暂时性错误（可重试）和系统性错误（如显存泄漏）
6. 每请求成本(Cost per Request)
   1. 定义：单次请求消耗的计算资源成本（如GPU秒数）
   2. 合理性：直接关联商业可行性，尤其在面向C端的高频调用场景（如AI写作助手）

## 2. 设计测试任务与性能指标选择
### 2.1 性能指标选择
1. 输出吞吐量(Tokens per Second)：可直接通过llama-bench的输出查看
2. 显存占用(Memory Use): 运行llama-bench时，在另一个终端使用指令`nvidia-smi --query-gpu=timestamp,utilization.gpu,utilization.memory,memory.total,memory.used --format=csv -l 1 > gpu_log.csv`，其中的`memory.used`便是显存占用
   
### 2.2 测试任务
使用llama-bench进行测试(llama-bench的详细使用请参考：https://github.com/ggml-org/llama.cpp/tree/master/tools/llama-bench)
1. 吞吐能力测试: 
   1. 不同batch_size下的吞吐能力测试
      1. 指令`./llama-bench -n 0 -p 1024 -b 128,256,512,1024 -m ./models/ggml-org_Qwen3-1.7B-GGUF_Qwen3-1.7B-Q4_K_M.gguf`;
      2. 其中的`-b 128,256,512,1024`用于测试不同batch_size的性能效果；
      3. `-n 0 -p 1024`指不生成token, 提示词长度为1024，这是常见的前向性能基准测试方法，常用于模型部署时评估吞吐能力   
   2. 不同threads_num下的吞吐能力测试
      1. `./llama-bench -n 0 -n 16 -p 64 -t 1,2,4,8,16,32 -m ./models/ggml-org_Qwen3-1.7B-GGUF_Qwen3-1.7B-Q4_K_M.gguf`
      2. `-t 1,2,4,8,16`即指定不同的线程数量
      3. 使用 prompt 长度 64（-p 64）
      4. 不生成 tokens 的测试：-n 0 → pp64
      5. 生成 16 tokens 的测试：-n 16 → tg16


## 3. 测试数据
### 3.1 吞吐能力测试
### 3.1.1 不同batch-size下的吞吐能力测试
`./llama-bench -n 0 -p 1024 -b 128,256,512,1024 -m ./models/ggml-org_Qwen3-1.7B-GGUF_Qwen3-1.7B-Q4_K_M.gguf`
| model                          |       size |     params | backend    | ngl | n_batch |            test |                  t/s |
| ------------------------------ | ---------: | ---------: | ---------- | --: | ------: | --------------: | -------------------: |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |     128 |          pp1024 |      2457.54 ± 79.48 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |     256 |          pp1024 |      2419.84 ± 30.16 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |     512 |          pp1024 |      2375.89 ± 16.96 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |    1024 |          pp1024 |      2408.87 ± 10.85 |

### 3.1.2 不同threads_num下的吞吐能力测试
1. pp<num>	prompt processing, prompt 长度 = num, 仅 prompt 前向速度
2. tg<num>	text generation, gen token 数 = num, prompt + token 生成速度
| model                          |       size |     params | backend    | ngl | threads |            test |                  t/s |
| ------------------------------ | ---------: | ---------: | ---------- | --: | ------: | --------------: | -------------------: |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |       1 |            pp64 |    2586.35 ± 1034.14 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |       1 |            tg16 |         34.37 ± 3.76 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |       2 |            pp64 |      2167.59 ± 42.58 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |       2 |            tg16 |         33.36 ± 0.34 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |       4 |            pp64 |      2164.58 ± 78.53 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |       4 |            tg16 |         33.73 ± 2.35 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |       8 |            pp64 |      2146.30 ± 55.10 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |       8 |            tg16 |         33.07 ± 1.15 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |      16 |            pp64 |      2084.91 ± 50.26 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |      16 |            tg16 |         32.79 ± 1.14 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |      32 |            pp64 |      2090.97 ± 47.05 |
| qwen3 1.7B Q4_K - Medium       |   1.19 GiB |     2.03 B | CUDA       |  99 |      32 |            tg16 |         32.79 ± 1.37 |

