# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller spec file for building RZEM AI Inference standalone executable.

This bundles Python + all dependencies (except models) into a single executable.
Models are downloaded from HuggingFace Hub on first run.

Build commands:
    Windows: pyinstaller build.spec
    macOS:   pyinstaller build.spec -- --windowed
    Linux:   pyinstaller build.spec
"""

import sys
from pathlib import Path

block_cipher = None

# Define paths
src_python = Path('src-python')
dist_frontend = Path('dist')

# Collect all Python source files
python_sources = [
    (str(src_python / 'main.py'), '.'),
    (str(src_python / 'api.py'), '.'),
    (str(src_python / 'app_state.py'), '.'),
    (str(src_python / 'events.py'), '.'),
]

# Collect Python packages
python_packages = ['inference', 'queue', 'db', 'shared', 'models', 'settings', 'utils']
for pkg in python_packages:
    pkg_path = src_python / pkg
    if pkg_path.exists():
        python_sources.append((str(pkg_path), pkg))

# Hidden imports for PyTorch and other dynamic imports
hidden_imports = [
    'torch',
    'torchvision',
    'diffusers',
    'transformers',
    'accelerate',
    'safetensors',
    'PIL',
    'PIL.Image',
    'PIL.ImageDraw',
    'PIL.ImageFont',
    'aiosqlite',
    'aiohttp',
    'websockets',
    'pydantic',
    'pydantic_settings',
    'loguru',
    'tqdm',
    'psutil',
    'pywebview',
    # PyTorch backends
    'torch.nn',
    'torch.cuda',
    'torch.backends.cuda',
    'torch.backends.cudnn',
    'torch.backends.mps',
    # Diffusers components
    'diffusers.models',
    'diffusers.pipelines',
    'diffusers.schedulers',
    # HuggingFace
    'huggingface_hub',
    'huggingface_hub.utils',
]

# Binary dependencies (PyTorch CUDA libs, etc.)
binaries = []

# Data files
datas = [
    # Frontend build
    (str(dist_frontend), 'dist'),
    # README and docs
    ('README.md', '.'),
    ('PYTHON_BACKEND_README.md', '.'),
    ('MIGRATION_GUIDE.md', '.'),
]

a = Analysis(
    [str(src_python / 'main.py')],
    pathex=[str(src_python)],
    binaries=binaries,
    datas=datas,
    hiddenimports=hidden_imports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        # Exclude unnecessary packages to reduce size
        'matplotlib',
        'scipy',
        'pandas',
        'jupyter',
        'notebook',
        'ipython',
        'pytest',
        'sphinx',
    ],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name='RZEM-AI-Inference',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,  # Set to False for windowed mode (no console)
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=None,  # TODO: Add application icon
)

# macOS app bundle
if sys.platform == 'darwin':
    app = BUNDLE(
        exe,
        name='RZEM AI Inference.app',
        icon=None,
        bundle_identifier='com.rzem.ai.inference',
        info_plist={
            'NSPrincipalClass': 'NSApplication',
            'NSHighResolutionCapable': 'True',
            'CFBundleShortVersionString': '0.1.0',
        },
    )
