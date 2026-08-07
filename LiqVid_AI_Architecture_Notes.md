# LiqVid AI Assistant - Architecture & Development Log

## Project Overview
Development notes for integrating a local Small Language Model (SLM) into **LiqVid**, a custom no-code Computational Fluid Dynamics (CFD) solver and visualizer written in Rust (using the PISO algorithm for STL geometries).

## 1. Local AI Inference on CPU
* **Feasibility:** Running SLMs locally on a CPU (like an 8th Gen i5) is highly viable using `llama.cpp` and **GGUF** quantization (compressing models to 4-bit precision).
* **Performance:** 1B-3B parameter models (like Qwen 2.5 Coder 1.5B or Llama 3.2 3B) can comfortably generate text faster than reading speed without a dedicated GPU.
* **Training Limitations:** Training or fine-tuning on a CPU takes exponentially longer (weeks/months) due to its serial processing architecture. Cloud GPUs (like a RunPod RTX 4090) are required if model weights need to be updated.

## 2. RAG vs. Fine-Tuning for a Codebase Assistant
For an actively developed software project like LiqVid, **Retrieval-Augmented Generation (RAG)** is the superior approach over fine-tuning:
* **Accuracy:** RAG grounds the AI in the exact, up-to-date `.rs` source code and `.md` documentation, preventing hallucinations of non-existent CFD functions.
* **Maintenance:** Re-indexing a changed file takes seconds, whereas re-training a fine-tuned model takes hours of compute time.
* **Offline Capability:** A RAG system can be bundled to run completely offline without API calls.

## 3. The Local RAG Pipeline Architecture
* **Ingestion:** A script parses the Rust codebase (`.rs`, `.toml`) and documentation (`.md`), chunks it, and converts it into mathematical vectors using an embedding model (like `all-MiniLM-L6-v2`).
* **Storage:** These vectors are stored in a local vector database (like `liqvid_db`).
* **Retrieval:** When a user asks a physics or usage question (e.g., "How do I fix a Courant number crash?"), the system fetches the relevant chunks of Rust code/docs and injects them into the prompt for the SLM to read and answer.

## 4. Cross-Platform Rust Deployment
To maintain LiqVid's seamless "no-code" experience for end-users across Linux, macOS, and Windows:
* **Embedded Inference:** Use Rust crates like `llama-cpp-2` and `candle` to run the model directly inside the Rust binary. No Python or Ollama installation is required by the user.
* **Bundled Database:** Compile the pre-indexed `liqvid_db` directly into the binary using the `include_bytes!` macro so users instantly have the RAG knowledge base.
* **Auto-Download Weights:** Write a startup function using the `directories` crate to automatically download the large `.gguf` SLM file to the correct OS-specific data folder on the first run.
* **GUI Integration:** `egui` is recommended for native 3D/Rust GUI integration to render the chat window alongside the STL visualizer.

## 5. CFD Memory Profiling Notes (OpenFOAM Context)
* The standard OpenFOAM `icoFoam/elbow` tutorial is a coarse 2D mesh (under 1,000 cells) and requires less than 30 MB RAM. 
* Memory overhead spikes significantly during 3D mesh generation (like `snappyHexMesh` in the `motorBike` tutorial), which can consume 1.5 GB to 2 GB for just 500,000 cells. LiqVid's internal meshing strategy should account for this overhead.

## 6. Code Sharing Methods
To share the private LiqVid codebase for further debugging or documentation generation:
1. **Flatten the repository:** Run `find . -name "*.rs" -exec echo -e "\n\n--- {} ---\n" \; -exec cat {} \; > liqvid_source.txt` and upload the text file.
2. **File-by-File:** Upload individual `.rs` or `Cargo.toml` files directly.
3. **Raw Links:** If the repo becomes public, provide the raw GitHub content URLs.
