use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// Opaque runtime model container used by the Rust port of InvokeAI's model manager.
/// In Python this was a large union of torch and diffusers types. Here we keep it
/// open-ended so higher layers can downcast to concrete implementations as needed.
pub type AnyModel = Arc<dyn Any + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseModelType {
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "sd-1")]
    StableDiffusion1,
    #[serde(rename = "sd-2")]
    StableDiffusion2,
    #[serde(rename = "sd-3")]
    StableDiffusion3,
    #[serde(rename = "sdxl")]
    StableDiffusionXL,
    #[serde(rename = "sdxl-refiner")]
    StableDiffusionXLRefiner,
    #[serde(rename = "flux")]
    Flux,
    #[serde(rename = "flux2")]
    Flux2,
    #[serde(rename = "cogview4")]
    CogView4,
    #[serde(rename = "z-image")]
    ZImage,
    #[serde(rename = "unknown")]
    Unknown,
}

impl Default for BaseModelType {
    fn default() -> Self {
        BaseModelType::Any
    }
}

impl BaseModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BaseModelType::Any => "any",
            BaseModelType::StableDiffusion1 => "sd-1",
            BaseModelType::StableDiffusion2 => "sd-2",
            BaseModelType::StableDiffusion3 => "sd-3",
            BaseModelType::StableDiffusionXL => "sdxl",
            BaseModelType::StableDiffusionXLRefiner => "sdxl-refiner",
            BaseModelType::Flux => "flux",
            BaseModelType::Flux2 => "flux2",
            BaseModelType::CogView4 => "cogview4",
            BaseModelType::ZImage => "z-image",
            BaseModelType::Unknown => "unknown",
        }
    }
}

impl fmt::Display for BaseModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    #[serde(rename = "onnx")]
    ONNX,
    #[serde(rename = "main")]
    Main,
    #[serde(rename = "vae")]
    VAE,
    #[serde(rename = "lora")]
    LoRA,
    #[serde(rename = "control_lora")]
    ControlLoRa,
    #[serde(rename = "controlnet")]
    ControlNet,
    #[serde(rename = "embedding")]
    TextualInversion,
    #[serde(rename = "ip_adapter")]
    IPAdapter,
    #[serde(rename = "clip_vision")]
    CLIPVision,
    #[serde(rename = "clip_embed")]
    CLIPEmbed,
    #[serde(rename = "t2i_adapter")]
    T2IAdapter,
    #[serde(rename = "t5_encoder")]
    T5Encoder,
    #[serde(rename = "qwen3_encoder")]
    Qwen3Encoder,
    #[serde(rename = "spandrel_image_to_image")]
    SpandrelImageToImage,
    #[serde(rename = "siglip")]
    SigLIP,
    #[serde(rename = "flux_redux")]
    FluxRedux,
    #[serde(rename = "llava_onevision")]
    LlavaOnevision,
    #[serde(rename = "unknown")]
    Unknown,
}

impl ModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelType::ONNX => "onnx",
            ModelType::Main => "main",
            ModelType::VAE => "vae",
            ModelType::LoRA => "lora",
            ModelType::ControlLoRa => "control_lora",
            ModelType::ControlNet => "controlnet",
            ModelType::TextualInversion => "embedding",
            ModelType::IPAdapter => "ip_adapter",
            ModelType::CLIPVision => "clip_vision",
            ModelType::CLIPEmbed => "clip_embed",
            ModelType::T2IAdapter => "t2i_adapter",
            ModelType::T5Encoder => "t5_encoder",
            ModelType::Qwen3Encoder => "qwen3_encoder",
            ModelType::SpandrelImageToImage => "spandrel_image_to_image",
            ModelType::SigLIP => "siglip",
            ModelType::FluxRedux => "flux_redux",
            ModelType::LlavaOnevision => "llava_onevision",
            ModelType::Unknown => "unknown",
        }
    }
}

impl fmt::Display for ModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubModelType {
    #[serde(rename = "unet")]
    UNet,
    #[serde(rename = "transformer")]
    Transformer,
    #[serde(rename = "text_encoder")]
    TextEncoder,
    #[serde(rename = "text_encoder_2")]
    TextEncoder2,
    #[serde(rename = "text_encoder_3")]
    TextEncoder3,
    #[serde(rename = "tokenizer")]
    Tokenizer,
    #[serde(rename = "tokenizer_2")]
    Tokenizer2,
    #[serde(rename = "tokenizer_3")]
    Tokenizer3,
    #[serde(rename = "vae")]
    VAE,
    #[serde(rename = "vae_decoder")]
    VAEDecoder,
    #[serde(rename = "vae_encoder")]
    VAEEncoder,
    #[serde(rename = "scheduler")]
    Scheduler,
    #[serde(rename = "safety_checker")]
    SafetyChecker,
}

impl SubModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubModelType::UNet => "unet",
            SubModelType::Transformer => "transformer",
            SubModelType::TextEncoder => "text_encoder",
            SubModelType::TextEncoder2 => "text_encoder_2",
            SubModelType::TextEncoder3 => "text_encoder_3",
            SubModelType::Tokenizer => "tokenizer",
            SubModelType::Tokenizer2 => "tokenizer_2",
            SubModelType::Tokenizer3 => "tokenizer_3",
            SubModelType::VAE => "vae",
            SubModelType::VAEDecoder => "vae_decoder",
            SubModelType::VAEEncoder => "vae_encoder",
            SubModelType::Scheduler => "scheduler",
            SubModelType::SafetyChecker => "safety_checker",
        }
    }
}

impl fmt::Display for SubModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClipVariantType {
    #[serde(rename = "large")]
    L,
    #[serde(rename = "gigantic")]
    G,
}

impl ClipVariantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClipVariantType::L => "large",
            ClipVariantType::G => "gigantic",
        }
    }
}

impl fmt::Display for ClipVariantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelVariantType {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "inpaint")]
    Inpaint,
    #[serde(rename = "depth")]
    Depth,
}

impl ModelVariantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelVariantType::Normal => "normal",
            ModelVariantType::Inpaint => "inpaint",
            ModelVariantType::Depth => "depth",
        }
    }
}

impl fmt::Display for ModelVariantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FluxVariantType {
    #[serde(rename = "schnell")]
    Schnell,
    #[serde(rename = "dev")]
    Dev,
    #[serde(rename = "dev_fill")]
    DevFill,
}

impl FluxVariantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FluxVariantType::Schnell => "schnell",
            FluxVariantType::Dev => "dev",
            FluxVariantType::DevFill => "dev_fill",
        }
    }
}

impl fmt::Display for FluxVariantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Flux2VariantType {
    #[serde(rename = "klein_4b")]
    Klein4B,
    #[serde(rename = "klein_9b")]
    Klein9B,
    #[serde(rename = "klein_9b_base")]
    Klein9BBase,
}

impl Flux2VariantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Flux2VariantType::Klein4B => "klein_4b",
            Flux2VariantType::Klein9B => "klein_9b",
            Flux2VariantType::Klein9BBase => "klein_9b_base",
        }
    }
}

impl fmt::Display for Flux2VariantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Qwen3VariantType {
    #[serde(rename = "qwen3_4b")]
    Qwen3_4B,
    #[serde(rename = "qwen3_8b")]
    Qwen3_8B,
}

impl Qwen3VariantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Qwen3VariantType::Qwen3_4B => "qwen3_4b",
            Qwen3VariantType::Qwen3_8B => "qwen3_8b",
        }
    }
}

impl fmt::Display for Qwen3VariantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelFormat {
    #[serde(rename = "omi")]
    OMI,
    #[serde(rename = "diffusers")]
    Diffusers,
    #[serde(rename = "checkpoint")]
    Checkpoint,
    #[serde(rename = "lycoris")]
    LyCORIS,
    #[serde(rename = "onnx")]
    ONNX,
    #[serde(rename = "olive")]
    Olive,
    #[serde(rename = "embedding_file")]
    EmbeddingFile,
    #[serde(rename = "embedding_folder")]
    EmbeddingFolder,
    #[serde(rename = "invokeai")]
    InvokeAI,
    #[serde(rename = "t5_encoder")]
    T5Encoder,
    #[serde(rename = "qwen3_encoder")]
    Qwen3Encoder,
    #[serde(rename = "bnb_quantized_int8b")]
    BnbQuantizedLlmInt8b,
    #[serde(rename = "bnb_quantized_nf4b")]
    BnbQuantizednf4b,
    #[serde(rename = "gguf_quantized")]
    GGUFQuantized,
    #[serde(rename = "unknown")]
    Unknown,
}

impl ModelFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelFormat::OMI => "omi",
            ModelFormat::Diffusers => "diffusers",
            ModelFormat::Checkpoint => "checkpoint",
            ModelFormat::LyCORIS => "lycoris",
            ModelFormat::ONNX => "onnx",
            ModelFormat::Olive => "olive",
            ModelFormat::EmbeddingFile => "embedding_file",
            ModelFormat::EmbeddingFolder => "embedding_folder",
            ModelFormat::InvokeAI => "invokeai",
            ModelFormat::T5Encoder => "t5_encoder",
            ModelFormat::Qwen3Encoder => "qwen3_encoder",
            ModelFormat::BnbQuantizedLlmInt8b => "bnb_quantized_int8b",
            ModelFormat::BnbQuantizednf4b => "bnb_quantized_nf4b",
            ModelFormat::GGUFQuantized => "gguf_quantized",
            ModelFormat::Unknown => "unknown",
        }
    }
}

impl fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchedulerPredictionType {
    #[serde(rename = "epsilon")]
    Epsilon,
    #[serde(rename = "v_prediction")]
    VPrediction,
    #[serde(rename = "sample")]
    Sample,
}

impl SchedulerPredictionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchedulerPredictionType::Epsilon => "epsilon",
            SchedulerPredictionType::VPrediction => "v_prediction",
            SchedulerPredictionType::Sample => "sample",
        }
    }
}

impl fmt::Display for SchedulerPredictionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelRepoVariant {
    #[serde(rename = "")] // Default variant is empty string in Python version
    Default,
    #[serde(rename = "fp16")]
    FP16,
    #[serde(rename = "fp32")]
    FP32,
    #[serde(rename = "onnx")]
    ONNX,
    #[serde(rename = "openvino")]
    OpenVINO,
    #[serde(rename = "flax")]
    Flax,
}

impl ModelRepoVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelRepoVariant::Default => "",
            ModelRepoVariant::FP16 => "fp16",
            ModelRepoVariant::FP32 => "fp32",
            ModelRepoVariant::ONNX => "onnx",
            ModelRepoVariant::OpenVINO => "openvino",
            ModelRepoVariant::Flax => "flax",
        }
    }
}

impl fmt::Display for ModelRepoVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelSourceType {
    #[serde(rename = "path")]
    Path,
    #[serde(rename = "url")]
    Url,
    #[serde(rename = "hf_repo_id")]
    HFRepoID,
}

impl ModelSourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelSourceType::Path => "path",
            ModelSourceType::Url => "url",
            ModelSourceType::HFRepoID => "hf_repo_id",
        }
    }
}

impl fmt::Display for ModelSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FluxLoRAFormat {
    #[serde(rename = "flux.diffusers")]
    Diffusers,
    #[serde(rename = "flux.kohya")]
    Kohya,
    #[serde(rename = "flux.onetrainer")]
    OneTrainer,
    #[serde(rename = "flux.control")]
    Control,
    #[serde(rename = "flux.aitoolkit")]
    AIToolkit,
    #[serde(rename = "flux.xlabs")]
    XLabs,
}

impl FluxLoRAFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            FluxLoRAFormat::Diffusers => "flux.diffusers",
            FluxLoRAFormat::Kohya => "flux.kohya",
            FluxLoRAFormat::OneTrainer => "flux.onetrainer",
            FluxLoRAFormat::Control => "flux.control",
            FluxLoRAFormat::AIToolkit => "flux.aitoolkit",
            FluxLoRAFormat::XLabs => "flux.xlabs",
        }
    }
}

impl fmt::Display for FluxLoRAFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnyVariant {
    Model(ModelVariantType),
    Clip(ClipVariantType),
    Flux(FluxVariantType),
    Flux2(Flux2VariantType),
    Qwen3(Qwen3VariantType),
}

impl fmt::Display for AnyVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            AnyVariant::Model(v) => v.as_str(),
            AnyVariant::Clip(v) => v.as_str(),
            AnyVariant::Flux(v) => v.as_str(),
            AnyVariant::Flux2(v) => v.as_str(),
            AnyVariant::Qwen3(v) => v.as_str(),
        };
        write!(f, "{}", value)
    }
}

impl From<ModelVariantType> for AnyVariant {
    fn from(value: ModelVariantType) -> Self {
        AnyVariant::Model(value)
    }
}

impl From<ClipVariantType> for AnyVariant {
    fn from(value: ClipVariantType) -> Self {
        AnyVariant::Clip(value)
    }
}

impl From<FluxVariantType> for AnyVariant {
    fn from(value: FluxVariantType) -> Self {
        AnyVariant::Flux(value)
    }
}

impl From<Flux2VariantType> for AnyVariant {
    fn from(value: Flux2VariantType) -> Self {
        AnyVariant::Flux2(value)
    }
}

impl From<Qwen3VariantType> for AnyVariant {
    fn from(value: Qwen3VariantType) -> Self {
        AnyVariant::Qwen3(value)
    }
}
