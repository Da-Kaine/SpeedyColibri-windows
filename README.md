# SpeedyColibri

A fast Rust Mixture-of-Experts (MoE) inference engine designed to stream large AI models efficiently on limited RAM/VRAM hardware.

> **Note:** For the full original documentation, architecture details, benchmarks, and performance tables, see [README_ORIGINAL.md](README_ORIGINAL.md).

---

## 🚀 Quick Start

### 1. Download & Install Engine

#### Windows

##### Option A: Pre-compiled Binary
Download the latest `coli.exe` binary directly from the [GitHub Releases](https://github.com/GriffinPilz/SpeedyColibri/releases) page, or run the installer script in PowerShell:

```powershell
.\scripts\install.ps1
```

##### Option B: Build from Source
If you have [Rust](https://rustup.rs) installed:

```powershell
# Build standard CPU release
.\scripts\build.ps1

# (Optional) Build with CUDA GPU support
$env:COLI_CUDA=1
.\scripts\build.ps1
```
The compiled binary will be located at `.\target\release\coli.exe`.

#### Linux / macOS

##### Option A: Installer Script
Run the installation script in terminal:

```bash
curl -sSL https://raw.githubusercontent.com/GriffinPilz/SpeedyColibri/main/scripts/install.sh | bash
```

##### Option B: Build from Source
```bash
# Build standard release
cargo build --release

# (Optional) Build with CUDA GPU support
cargo build --release --features cuda
```
The compiled binary will be located at `./target/release/coli`.

---

### 2. Download Model Container

Download ready-to-run model containers directly using `scripts/fetch.sh`:

```bash
# Download a specific model container (e.g. maple-preview)
./scripts/fetch.sh maple-preview

# List available models in the registry
python3 scripts/model.py list
```

Available models include `maple-preview`, `nemotron-3-super`, `minimax-m2.7`, `deepseek-v4-flash`, `minimax-m3`, `glm-5.2`, and `kimi-k3`.

---

## 🌐 Starting the Web GUI / API Server

`SpeedyColibri` provides an OpenAI-compatible HTTP API server that powers chat interfaces and Web GUIs.

### Step 1: Start the Inference Server

Run `coli.exe serve` specifying the model directory or model tag and port (default is `8080`):

```powershell
# Fetch model container (if needed) and start the server on port 8080
.\target\release\coli.exe serve maple-preview 8080
```

The server exposes OpenAI-compatible REST endpoints:
* **Base URL:** `http://localhost:8080/v1`
* **Health Check:** `http://localhost:8080/health`
* **Chat Completions:** `http://localhost:8080/v1/chat/completions`

### Step 2: Connect a Web GUI

Because `SpeedyColibri` serves standard OpenAI API endpoints, you can connect any popular OpenAI-compatible frontend:

#### Open WebUI
1. Run Open WebUI (via Docker or local executable).
2. Set `OPENAI_API_BASE_URL` to `http://localhost:8080/v1` (or `http://host.docker.internal:8080/v1` if using Docker).
3. Set `OPENAI_API_KEY` to any dummy value (e.g. `colibri`).

#### Chatbot UI / Lobe Chat / Other Web Frontends
Configure the custom OpenAI API endpoint in settings:
* **API Key:** `colibri` (or any string)
* **API Host / Base URL:** `http://localhost:8080/v1`

### Step 3: Test with cURL

You can also send a request directly from PowerShell or Command Prompt:

```bash
curl http://localhost:8080/v1/chat/completions `
  -H "Content-Type: application/json" `
  -d '{
    "messages": [{"role": "user", "content": "Hello! Explain quantum computing in one sentence."}],
    "max_tokens": 64
  }'
```

---

## 📚 More Information

* Full Benchmarks & Engine Details: [README_ORIGINAL.md](README_ORIGINAL.md)
* Configuration Guide: [docs/CONFIGURATION.md](docs/CONFIGURATION.md)
* Development & Porting Status: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)
