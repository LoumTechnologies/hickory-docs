//! Local LLM inference via llama.cpp (GGUF models).
//!
//! Runs inference entirely in-process — no external server needed.
//! Enable with the `local` feature flag. Metal is auto-enabled on macOS.
//!
//! On first run, automatically calibrates GPU vs CPU by benchmarking both
//! and caching the result in `.hick/calibration.json`.

use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use futures::Stream;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use serde::{Deserialize, Serialize};

use crate::llm::{ChatChunk, LlmClient, Message, Role};

/// Local LLM client using llama.cpp for inference.
///
/// Loads a GGUF model and runs inference entirely in-process.
/// Token generation runs on a blocking thread to avoid stalling the async runtime.
pub struct LocalLlmClient {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    model_name: String,
    n_ctx: u32,
    max_tokens: u32,
    backend_label: String,
}

/// Cached calibration result.
#[derive(Serialize, Deserialize)]
struct CalibrationCache {
    model_file: String,
    model_size_bytes: u64,
    n_gpu_layers: u32,
    backend: String,
    tokens_per_sec: f64,
}

/// Which GPU backend was compiled in.
fn compiled_gpu_backend() -> Option<&'static str> {
    if cfg!(target_os = "macos") {
        Some("Metal")
    } else if cfg!(feature = "local-cuda") {
        Some("CUDA")
    } else {
        None
    }
}

impl LocalLlmClient {
    /// Load with explicit GPU layer setting (no calibration).
    pub fn load(
        model_path: impl Into<PathBuf>,
        n_gpu_layers: u32,
        n_ctx: u32,
        max_tokens: u32,
    ) -> anyhow::Result<Self> {
        let model_path = model_path.into();
        let backend = LlamaBackend::init()
            .map_err(|e| anyhow::anyhow!("Failed to init llama backend: {e}"))?;
        Self::load_with_backend(backend, &model_path, n_gpu_layers, n_ctx, max_tokens)
    }

    /// Load with automatic GPU/CPU calibration.
    ///
    /// On first run for a given model, benchmarks GPU (if compiled in) and CPU,
    /// picks the faster one, and caches the result in `project_dir/.hick/calibration.json`.
    pub fn load_auto(
        model_path: impl Into<PathBuf>,
        n_ctx: u32,
        max_tokens: u32,
        project_dir: &Path,
    ) -> anyhow::Result<Self> {
        let model_path = model_path.into();
        let cache_path = project_dir.join(".hick").join("calibration.json");

        // Check calibration cache
        if let Some(cached) = load_cache(&cache_path, &model_path) {
            eprintln!(
                "Using cached config: {} ({:.1} tok/s)",
                cached.backend, cached.tokens_per_sec
            );
            let backend = LlamaBackend::init()
                .map_err(|e| anyhow::anyhow!("Failed to init llama backend: {e}"))?;
            return Self::load_with_backend(
                backend,
                &model_path,
                cached.n_gpu_layers,
                n_ctx,
                max_tokens,
            );
        }

        // Run calibration
        let backend = LlamaBackend::init()
            .map_err(|e| anyhow::anyhow!("Failed to init llama backend: {e}"))?;
        let (n_gpu_layers, backend_name, tps) = calibrate(&backend, &model_path, n_ctx)?;

        // Cache result
        save_cache(&cache_path, &model_path, n_gpu_layers, &backend_name, tps);

        Self::load_with_backend(backend, &model_path, n_gpu_layers, n_ctx, max_tokens)
    }

    fn load_with_backend(
        backend: LlamaBackend,
        model_path: &Path,
        n_gpu_layers: u32,
        n_ctx: u32,
        max_tokens: u32,
    ) -> anyhow::Result<Self> {
        eprintln!("Loading model: {} ...", model_path.display());

        let model_params = LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers);
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
            .map_err(|e| anyhow::anyhow!("Failed to load model {}: {e}", model_path.display()))?;

        let model_name = model_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "local".into());

        let backend_label = if n_gpu_layers > 0 {
            compiled_gpu_backend().unwrap_or("GPU").to_string()
        } else {
            "CPU".to_string()
        };

        eprintln!(
            "Model loaded: {} ({:.0} MB, {} params, {})",
            model_name,
            model.size() as f64 / 1_048_576.0,
            model.n_params(),
            backend_label,
        );

        Ok(Self {
            backend: Arc::new(backend),
            model: Arc::new(model),
            model_name,
            n_ctx,
            max_tokens,
            backend_label,
        })
    }
}

// ---------------------------------------------------------------------------
// Calibration
// ---------------------------------------------------------------------------

/// Benchmark a model configuration by generating ~20 tokens. Returns tok/s.
fn benchmark_config(
    backend: &LlamaBackend,
    model_path: &Path,
    n_ctx: u32,
    n_gpu_layers: u32,
) -> anyhow::Result<f64> {
    let params = LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers);
    let model = LlamaModel::load_from_file(backend, model_path, &params)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let ctx_params =
        LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(n_ctx).unwrap()));
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let prompt = "Write a Python function that prints hello world.";
    let tokens = model
        .str_to_token(prompt, AddBos::Always)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut batch = LlamaBatch::new(n_ctx as usize, 1);
    for (i, token) in tokens.iter().enumerate() {
        batch
            .add(*token, i as i32, &[0], i == tokens.len() - 1)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    ctx.decode(&mut batch).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut sampler = LlamaSampler::greedy();
    let n_generate = 20;
    let mut n_cur = tokens.len() as i32;
    let mut generated = 0;

    let start = Instant::now();
    for _ in 0..n_generate {
        let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(new_token);
        if model.is_eog_token(new_token) {
            break;
        }
        generated += 1;
        batch.clear();
        batch
            .add(new_token, n_cur, &[0], true)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        ctx.decode(&mut batch).map_err(|e| anyhow::anyhow!("{e}"))?;
        n_cur += 1;
    }
    let elapsed = start.elapsed();

    if generated == 0 {
        return Ok(0.0);
    }
    Ok(generated as f64 / elapsed.as_secs_f64())
}

/// Run calibration: benchmark GPU (if compiled in) and CPU, return the best config.
fn calibrate(
    backend: &LlamaBackend,
    model_path: &Path,
    n_ctx: u32,
) -> anyhow::Result<(u32, String, f64)> {
    eprintln!("Calibrating inference backends...");

    let mut gpu_tps = None;
    let gpu_name = compiled_gpu_backend().unwrap_or("GPU");

    // Benchmark GPU if a backend is compiled in
    if compiled_gpu_backend().is_some() {
        match benchmark_config(backend, model_path, n_ctx, 1000) {
            Ok(tps) => {
                eprintln!("  {gpu_name} : {tps:.1} tok/s");
                gpu_tps = Some(tps);
            }
            Err(e) => {
                eprintln!("  {gpu_name} : failed ({e})");
            }
        }
    }

    // Benchmark CPU
    let cpu_tps = match benchmark_config(backend, model_path, n_ctx, 0) {
        Ok(tps) => {
            eprintln!("  CPU   : {tps:.1} tok/s");
            Some(tps)
        }
        Err(e) => {
            eprintln!("  CPU   : failed ({e})");
            None
        }
    };

    match (gpu_tps, cpu_tps) {
        (Some(gpu), Some(cpu)) => {
            if gpu >= cpu {
                let speedup = gpu / cpu.max(0.01);
                eprintln!("  → Using {gpu_name} ({speedup:.1}x faster than CPU)");
                Ok((1000, gpu_name.to_string(), gpu))
            } else {
                let speedup = cpu / gpu.max(0.01);
                eprintln!("  → Using CPU ({speedup:.1}x faster than {gpu_name})");
                Ok((0, "CPU".into(), cpu))
            }
        }
        (Some(gpu), None) => {
            eprintln!("  → Using {gpu_name}");
            Ok((1000, gpu_name.to_string(), gpu))
        }
        (None, Some(cpu)) => {
            eprintln!("  → Using CPU");
            Ok((0, "CPU".into(), cpu))
        }
        (None, None) => {
            anyhow::bail!("Both GPU and CPU benchmarks failed. Check your model file.")
        }
    }
}

// ---------------------------------------------------------------------------
// Calibration cache
// ---------------------------------------------------------------------------

fn model_key(model_path: &Path) -> Option<(String, u64)> {
    let name = model_path.file_name()?.to_string_lossy().to_string();
    let size = std::fs::metadata(model_path).ok()?.len();
    Some((name, size))
}

fn load_cache(cache_path: &Path, model_path: &Path) -> Option<CalibrationCache> {
    let (name, size) = model_key(model_path)?;
    let data = std::fs::read_to_string(cache_path).ok()?;
    let cached: CalibrationCache = serde_json::from_str(&data).ok()?;
    if cached.model_file == name && cached.model_size_bytes == size {
        Some(cached)
    } else {
        None
    }
}

fn save_cache(cache_path: &Path, model_path: &Path, n_gpu_layers: u32, backend: &str, tps: f64) {
    let Some((name, size)) = model_key(model_path) else {
        return;
    };
    let cache = CalibrationCache {
        model_file: name,
        model_size_bytes: size,
        n_gpu_layers,
        backend: backend.to_string(),
        tokens_per_sec: tps,
    };
    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&cache) {
        let _ = std::fs::write(cache_path, json);
    }
}

// ---------------------------------------------------------------------------
// Prompt formatting
// ---------------------------------------------------------------------------

fn format_prompt(model: &LlamaModel, messages: &[Message]) -> anyhow::Result<String> {
    let chat_messages: Result<Vec<LlamaChatMessage>, _> = messages
        .iter()
        .map(|m| {
            let role = match m.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            LlamaChatMessage::new(role.to_string(), m.content.clone())
        })
        .collect();
    let chat_messages =
        chat_messages.map_err(|e| anyhow::anyhow!("Failed to create chat messages: {e}"))?;

    match model.chat_template(None) {
        Ok(template) => {
            let prompt = model
                .apply_chat_template(&template, &chat_messages, true)
                .map_err(|e| anyhow::anyhow!("Failed to apply chat template: {e}"))?;
            Ok(prompt)
        }
        Err(_) => {
            // Fallback for models without a chat template
            let mut prompt = String::new();
            for m in messages {
                match m.role {
                    Role::System => prompt.push_str(&format!("System: {}\n\n", m.content)),
                    Role::User => prompt.push_str(&format!("User: {}\n\n", m.content)),
                    Role::Assistant => prompt.push_str(&format!("Assistant: {}\n\n", m.content)),
                }
            }
            prompt.push_str("Assistant: ");
            Ok(prompt)
        }
    }
}

// ---------------------------------------------------------------------------
// Generation (blocking)
// ---------------------------------------------------------------------------

fn generate_blocking(
    model: &LlamaModel,
    backend: &LlamaBackend,
    messages: Vec<Message>,
    n_ctx: u32,
    max_tokens: u32,
) -> anyhow::Result<String> {
    let prompt = format_prompt(model, &messages)?;
    let tokens = model
        .str_to_token(&prompt, AddBos::Always)
        .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

    if tokens.len() as u32 >= n_ctx {
        anyhow::bail!(
            "Prompt ({} tokens) exceeds context window ({n_ctx}). \
             Use a larger --n-ctx or shorten the conversation.",
            tokens.len()
        );
    }

    let ctx_params =
        LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(n_ctx).unwrap()));
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| anyhow::anyhow!("Failed to create context: {e}"))?;

    let mut batch = LlamaBatch::new(n_ctx as usize, 1);
    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;
        batch
            .add(*token, i as i32, &[0], is_last)
            .map_err(|e| anyhow::anyhow!("Batch add failed: {e}"))?;
    }

    ctx.decode(&mut batch)
        .map_err(|e| anyhow::anyhow!("Prompt decode failed: {e}"))?;

    let mut sampler = LlamaSampler::greedy();
    let mut output = String::new();
    let mut n_cur = tokens.len() as i32;

    for _ in 0..max_tokens {
        let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(new_token);

        if model.is_eog_token(new_token) {
            break;
        }

        let token_bytes = model
            .token_to_piece_bytes(new_token, 64, true, None)
            .map_err(|e| anyhow::anyhow!("Token decode failed: {e}"))?;
        let token_str = String::from_utf8_lossy(&token_bytes).into_owned();
        output.push_str(&token_str);

        batch.clear();
        batch
            .add(new_token, n_cur, &[0], true)
            .map_err(|e| anyhow::anyhow!("Batch add failed: {e}"))?;
        ctx.decode(&mut batch)
            .map_err(|e| anyhow::anyhow!("Decode failed: {e}"))?;
        n_cur += 1;
    }

    Ok(output)
}

fn generate_streaming(
    model: &LlamaModel,
    backend: &LlamaBackend,
    messages: Vec<Message>,
    n_ctx: u32,
    max_tokens: u32,
    tx: &tokio::sync::mpsc::Sender<anyhow::Result<ChatChunk>>,
) -> anyhow::Result<()> {
    let prompt = format_prompt(model, &messages)?;
    let tokens = model
        .str_to_token(&prompt, AddBos::Always)
        .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

    if tokens.len() as u32 >= n_ctx {
        anyhow::bail!(
            "Prompt ({} tokens) exceeds context window ({n_ctx})",
            tokens.len()
        );
    }

    let ctx_params =
        LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(n_ctx).unwrap()));
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| anyhow::anyhow!("Failed to create context: {e}"))?;

    let mut batch = LlamaBatch::new(n_ctx as usize, 1);
    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;
        batch
            .add(*token, i as i32, &[0], is_last)
            .map_err(|e| anyhow::anyhow!("Batch add failed: {e}"))?;
    }

    ctx.decode(&mut batch)
        .map_err(|e| anyhow::anyhow!("Prompt decode failed: {e}"))?;

    let mut sampler = LlamaSampler::greedy();
    let mut n_cur = tokens.len() as i32;

    for _ in 0..max_tokens {
        let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(new_token);

        if model.is_eog_token(new_token) {
            break;
        }

        let token_bytes = model
            .token_to_piece_bytes(new_token, 64, true, None)
            .map_err(|e| anyhow::anyhow!("Token decode failed: {e}"))?;
        let token_str = String::from_utf8_lossy(&token_bytes).into_owned();

        // Send token to the stream. If receiver is dropped, stop generating.
        if tx
            .blocking_send(Ok(ChatChunk {
                delta: token_str,
                finish_reason: None,
            }))
            .is_err()
        {
            break;
        }

        batch.clear();
        batch
            .add(new_token, n_cur, &[0], true)
            .map_err(|e| anyhow::anyhow!("Batch add failed: {e}"))?;
        ctx.decode(&mut batch)
            .map_err(|e| anyhow::anyhow!("Decode failed: {e}"))?;
        n_cur += 1;
    }

    // Signal completion
    let _ = tx.blocking_send(Ok(ChatChunk {
        delta: String::new(),
        finish_reason: Some("stop".into()),
    }));

    Ok(())
}

// ---------------------------------------------------------------------------
// LlmClient implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl LlmClient for LocalLlmClient {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let model = Arc::clone(&self.model);
        let backend = Arc::clone(&self.backend);
        let n_ctx = self.n_ctx;
        let max_tokens = self.max_tokens;

        tokio::task::spawn_blocking(move || {
            generate_blocking(&model, &backend, messages, n_ctx, max_tokens)
        })
        .await?
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        let model = Arc::clone(&self.model);
        let backend = Arc::clone(&self.backend);
        let n_ctx = self.n_ctx;
        let max_tokens = self.max_tokens;

        tokio::task::spawn_blocking(move || {
            if let Err(e) = generate_streaming(&model, &backend, messages, n_ctx, max_tokens, &tx) {
                let _ = tx.blocking_send(Err(e));
            }
        });

        let stream = futures::stream::unfold(rx, |mut rx| async {
            rx.recv().await.map(|item| (item, rx))
        });

        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        &self.backend_label
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }
}
