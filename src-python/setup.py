"""Setup script for rzem-ai-inference Python backend"""

from setuptools import setup, find_packages

setup(
    name="rzem-ai-inference",
    version="0.1.0",
    description="Local AI Image Generation with GPU Sharing",
    author="RZEM AI",
    packages=find_packages(),
    install_requires=[
        "torch>=2.0.0",
        "torchvision>=0.15.0",
        "diffusers>=0.32.0",
        "transformers>=4.35.0",
        "accelerate>=0.24.0",
        "safetensors>=0.4.0",
        "huggingface-hub>=0.25.0",
        "pywebview>=5.0.0",
        "Pillow>=10.0.0",
        "opencv-python>=4.8.0",
        "aiosqlite>=0.19.0",
        "aiohttp>=3.9.0",
        "websockets>=12.0",
        "python-dotenv>=1.0.0",
        "pydantic>=2.0.0",
        "pydantic-settings>=2.0.0",
        "click>=8.1.0",
        "aiofiles>=23.0.0",
        "loguru>=0.7.0",
        "tqdm>=4.66.0",
        "jinja2>=3.1.0",
        "psutil>=5.9.0",
    ],
    python_requires=">=3.10",
    entry_points={
        "console_scripts": [
            "rzem-ai=main:main",
        ],
    },
)
