//! Phi-3 Model loading and inference
//!
//! This module handles loading the Phi-3-mini-4k-instruct model from HuggingFace
//! and provides inference capabilities with GPU auto-detection.

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::phi3::{Config as Phi3Config, Model as Phi3};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;
use std::path::PathBuf;

/// Configuration for model loading
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Model ID on HuggingFace Hub
    pub model_id: String,
    /// Revision/branch to use
    pub revision: String,
    /// Use quantized model for lower memory usage
    pub use_quantized: bool,
    /// Maximum context length
    pub max_context_length: usize,
    /// Temperature for sampling (0.0 = greedy, higher = more random)
    pub temperature: f64,
    /// Top-p sampling parameter
    pub top_p: f64,
    /// Repetition penalty
    pub repeat_penalty: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_id: "microsoft/Phi-3-mini-4k-instruct".to_string(),
            revision: "main".to_string(),
            use_quantized: true,
            max_context_length: 4096,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
        }
    }
}

/// Result type for model operations
pub type ModelResult<T> = Result<T, ModelError>;

/// Error types for model operations
#[derive(Debug)]
pub enum ModelError {
    /// Failed to detect or initialize device
    DeviceError(String),
    /// Failed to download model from HuggingFace
    DownloadError(String),
    /// Failed to load model weights
    LoadError(String),
    /// Failed during inference
    InferenceError(String),
    /// Tokenization error
    TokenizationError(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::DeviceError(msg) => write!(f, "Device error: {}", msg),
            ModelError::DownloadError(msg) => write!(f, "Download error: {}", msg),
            ModelError::LoadError(msg) => write!(f, "Load error: {}", msg),
            ModelError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            ModelError::TokenizationError(msg) => write!(f, "Tokenization error: {}", msg),
        }
    }
}

impl std::error::Error for ModelError {}

/// The Phi-3 model wrapper for browser AI agent
pub struct Phi3Model {
    model: Phi3,
    tokenizer: Tokenizer,
    device: Device,
    config: ModelConfig,
    logits_processor: LogitsProcessor,
}

impl Phi3Model {
    /// Create a new Phi-3 model instance
    ///
    /// This will:
    /// 1. Auto-detect the best available device (CUDA > Metal > CPU)
    /// 2. Download model files from HuggingFace Hub
    /// 3. Load the model into memory
    pub fn new(config: ModelConfig) -> ModelResult<Self> {
        log::info!("Initializing Phi-3 model...");

        // Detect best device
        let device = Self::detect_device()?;
        log::info!("Using device: {:?}", device);

        // Download model files
        let (model_path, tokenizer_path, config_path) = Self::download_model(&config)?;
        log::info!("Model files downloaded");

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| ModelError::TokenizationError(e.to_string()))?;
        log::info!("Tokenizer loaded");

        // Load model config
        let model_config: Phi3Config = serde_json::from_str(
            &std::fs::read_to_string(&config_path)
                .map_err(|e| ModelError::LoadError(e.to_string()))?,
        )
        .map_err(|e| ModelError::LoadError(format!("Failed to parse config: {}", e)))?;

        // Load model weights
        let vb = if config.use_quantized {
            Self::load_quantized_weights(&model_path, &device)?
        } else {
            Self::load_full_weights(&model_path, &device)?
        };

        let model = Phi3::new(&model_config, vb)
            .map_err(|e| ModelError::LoadError(e.to_string()))?;
        log::info!("Model loaded successfully");

        // Initialize logits processor for sampling
        let logits_processor = LogitsProcessor::new(
            42, // seed
            Some(config.temperature),
            Some(config.top_p),
        );

        Ok(Self {
            model,
            tokenizer,
            device,
            config,
            logits_processor,
        })
    }

    /// Detect the best available compute device
    pub fn detect_device() -> ModelResult<Device> {
        // Try CUDA first
        #[cfg(feature = "cuda")]
        {
            if let Ok(device) = Device::cuda(0) {
                log::info!("CUDA device detected");
                return Ok(device);
            }
        }

        // Try Metal (Apple Silicon)
        #[cfg(feature = "metal")]
        {
            if let Ok(device) = Device::new_metal(0) {
                log::info!("Metal device detected");
                return Ok(device);
            }
        }

        // Fall back to CPU
        log::info!("Falling back to CPU");
        Ok(Device::Cpu)
    }

    /// Download model files from HuggingFace Hub
    fn download_model(config: &ModelConfig) -> ModelResult<(PathBuf, PathBuf, PathBuf)> {
        let api = Api::new()
            .map_err(|e| ModelError::DownloadError(e.to_string()))?;

        let repo = api.repo(Repo::with_revision(
            config.model_id.clone(),
            RepoType::Model,
            config.revision.clone(),
        ));

        // Download required files
        let model_path = if config.use_quantized {
            repo.get("model.safetensors")
                .or_else(|_| repo.get("pytorch_model.bin"))
                .map_err(|e| ModelError::DownloadError(e.to_string()))?
        } else {
            repo.get("model.safetensors")
                .or_else(|_| repo.get("pytorch_model.bin"))
                .map_err(|e| ModelError::DownloadError(e.to_string()))?
        };

        let tokenizer_path = repo.get("tokenizer.json")
            .map_err(|e| ModelError::DownloadError(e.to_string()))?;

        let config_path = repo.get("config.json")
            .map_err(|e| ModelError::DownloadError(e.to_string()))?;

        Ok((model_path, tokenizer_path, config_path))
    }

    /// Load quantized model weights (lower memory usage)
    fn load_quantized_weights(path: &PathBuf, device: &Device) -> ModelResult<VarBuilder<'static>> {
        let weights = candle_core::safetensors::load(path, device)
            .map_err(|e| ModelError::LoadError(e.to_string()))?;
        
        Ok(VarBuilder::from_tensors(weights, DType::F16, device))
    }

    /// Load full precision model weights
    fn load_full_weights(path: &PathBuf, device: &Device) -> ModelResult<VarBuilder<'static>> {
        let weights = candle_core::safetensors::load(path, device)
            .map_err(|e| ModelError::LoadError(e.to_string()))?;
        
        Ok(VarBuilder::from_tensors(weights, DType::F32, device))
    }

    /// Generate text from a prompt
    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> ModelResult<String> {
        log::debug!("Generating response for prompt: {}...", &prompt[..prompt.len().min(50)]);

        // Tokenize the prompt
        let encoding = self.tokenizer
            .encode(prompt, true)
            .map_err(|e| ModelError::TokenizationError(e.to_string()))?;

        let mut tokens: Vec<u32> = encoding.get_ids().to_vec();
        let prompt_len = tokens.len();

        // Check context length
        if prompt_len >= self.config.max_context_length {
            return Err(ModelError::InferenceError(
                "Prompt exceeds maximum context length".to_string(),
            ));
        }

        let max_new_tokens = max_tokens.min(self.config.max_context_length - prompt_len);

        // Generate tokens
        for _ in 0..max_new_tokens {
            // Create input tensor
            let input = Tensor::new(&tokens[..], &self.device)
                .map_err(|e| ModelError::InferenceError(e.to_string()))?
                .unsqueeze(0)
                .map_err(|e| ModelError::InferenceError(e.to_string()))?;

            // Forward pass
            let logits = self.model
                .forward(&input, tokens.len())
                .map_err(|e| ModelError::InferenceError(e.to_string()))?;

            // Get logits for last token
            let logits = logits
                .squeeze(0)
                .map_err(|e| ModelError::InferenceError(e.to_string()))?;
            let logits = logits
                .get(logits.dim(0).map_err(|e| ModelError::InferenceError(e.to_string()))? - 1)
                .map_err(|e| ModelError::InferenceError(e.to_string()))?;

            // Apply repetition penalty
            let logits = self.apply_repetition_penalty(&logits, &tokens)?;

            // Sample next token
            let next_token = self.logits_processor
                .sample(&logits)
                .map_err(|e| ModelError::InferenceError(e.to_string()))?;

            // Check for EOS
            if self.is_eos_token(next_token) {
                break;
            }

            tokens.push(next_token);
        }

        // Decode generated tokens (excluding prompt)
        let generated_tokens = &tokens[prompt_len..];
        let output = self.tokenizer
            .decode(generated_tokens, true)
            .map_err(|e| ModelError::TokenizationError(e.to_string()))?;

        Ok(output)
    }

    /// Apply repetition penalty to logits
    fn apply_repetition_penalty(&self, logits: &Tensor, tokens: &[u32]) -> ModelResult<Tensor> {
        let mut logits_vec: Vec<f32> = logits
            .to_vec1()
            .map_err(|e| ModelError::InferenceError(e.to_string()))?;

        for &token in tokens {
            let token = token as usize;
            if token < logits_vec.len() {
                if logits_vec[token] > 0.0 {
                    logits_vec[token] /= self.config.repeat_penalty;
                } else {
                    logits_vec[token] *= self.config.repeat_penalty;
                }
            }
        }

        Tensor::new(logits_vec, &self.device)
            .map_err(|e| ModelError::InferenceError(e.to_string()))
    }

    /// Check if token is end-of-sequence
    fn is_eos_token(&self, token: u32) -> bool {
        // Phi-3 uses token 32000 as EOS
        // Also check for common EOS tokens
        token == 32000 || token == 32001 || token == 2
    }

    /// Get the device being used
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get the tokenizer
    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_detection() {
        let device = Phi3Model::detect_device();
        assert!(device.is_ok());
    }

    #[test]
    fn test_default_config() {
        let config = ModelConfig::default();
        assert_eq!(config.model_id, "microsoft/Phi-3-mini-4k-instruct");
        assert!(config.use_quantized);
    }
}
