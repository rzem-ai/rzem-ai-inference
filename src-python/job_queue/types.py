"""Queue types matching the Rust implementation"""

from enum import Enum
from typing import Optional, List
from datetime import datetime
from pydantic import BaseModel, Field
import uuid


class JobStatus(str, Enum):
    """Generation job status"""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class SamplerType(str, Enum):
    """Sampling algorithm types"""
    EULER = "euler"
    EULER_A = "euler_a"
    DPM_PP_2M = "dpm_pp_2m"


class SchedulerType(str, Enum):
    """Noise schedule types"""
    NORMAL = "normal"
    SIMPLE = "simple"
    KARRAS = "karras"
    EXPONENTIAL = "exponential"


class LoraConfig(BaseModel):
    """LoRA adapter configuration"""
    id: Optional[str] = None       # UUID from database (frontend sends this)
    path: Optional[str] = None     # Resolved file path (processor fills this)
    strength: float = 1.0
    name: Optional[str] = None


class GenerationParams(BaseModel):
    """Parameters for image generation"""
    prompt: str
    negative_prompt: Optional[str] = None
    steps: int = 4
    cfg_scale: float = 1.0
    width: int = 1024
    height: int = 1024
    seed: int = -1
    model_type: Optional[str] = None
    sampler: Optional[SamplerType] = None
    scheduler: Optional[SchedulerType] = None
    loras: List[LoraConfig] = Field(default_factory=list)
    bundle_id: Optional[str] = None
    model_component_id: str
    t5_component_id: str
    clip_component_id: str
    vae_component_id: str


class GenerationJob(BaseModel):
    """Generation job with status and metadata"""
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    params: GenerationParams
    status: JobStatus = JobStatus.PENDING
    progress: float = 0.0
    created_at: datetime = Field(default_factory=datetime.utcnow)
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    result_path: Optional[str] = None
    error: Optional[str] = None
    image_id: Optional[str] = None

    class Config:
        json_encoders = {
            datetime: lambda v: int(v.timestamp())
        }


class ProgressUpdate(BaseModel):
    """Progress update event"""
    job_id: str
    progress: float
    stage: Optional[str] = None
    current_step: Optional[int] = None
    total_steps: Optional[int] = None
