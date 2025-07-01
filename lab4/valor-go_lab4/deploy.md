# 部署说明文档

## Valor-go Team

## 1. 编译相关配置（强烈推荐和文档配置保持一致）
请根据自己的GPU等配置选择对应的CUDA版本和编译器版本,详细安装文档见llama.cpp官方文档和CUDA官方文档

1. 操作系统：Ubuntu 22.04
2. Driver Version: 550.163.01
3. CUDA Version: 12.4
4. gcc (Ubuntu 11.4.0-1ubuntu1~22.04) 11.4.0
5. g++ (Ubuntu 11.4.0-1ubuntu1~22.04) 11.4.0
6. CPU: AMD Ryzen 9 7945HX with Radeon Graphics
7. GPU:NVIDIA GeForce RTX 4060

说明：

1. Driver Version，CUDA Version，gcc，g++各版本需要保持一致，否则可能无法编译
2. 不建议使用clang，CUDA对clang支持不太好

## 2. 获取模型

1. 使用：Qwen3-1.7B
2. 指令：由于国内防火墙的问题，我们使用镜像来实现
   1. 在终端 `export HF_ENDPOINT=https://hf-mirror.com`配置hugging-face镜像访问
   2. cd 到 `.../llama.cpp/build/bin`然后使用指令 `./llama-server -hf ggml-org/Qwen3-1.7B-GGUF: Q4_K_M`即可获取模型并且打开端口进行对话
   3. `mv ~/.cache/llama.cpp/ggml-org_Qwen3-1.7B-GGUF_Qwen3-1.7B-Q4_K_M.gguf .../llama.cpp/build/bin/models/`将模型存储到buid/bin文件夹中方便使用（前面的路径是默认下载路径，可能不同机器会有所不同）
3. 部署成功截图：![deploy success](image/deploy/deploy_success.png)
