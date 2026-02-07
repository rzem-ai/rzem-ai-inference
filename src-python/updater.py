"""
Auto-update system for RZEM AI Inference.

This module handles checking for updates, downloading new versions,
and applying updates from GitHub releases.

Architecture:
- Check GitHub API for latest release
- Compare with current version (semantic versioning)
- Download update if available
- Verify integrity (SHA256 checksum)
- Apply update (replace executable, restart)

Usage:
    from updater import AutoUpdater

    updater = AutoUpdater()
    if await updater.check_for_updates():
        await updater.download_and_install()
"""

import asyncio
import hashlib
import json
import platform
import sys
from pathlib import Path
from typing import Optional, Dict, Any
from loguru import logger
import aiohttp
import aiofiles

# Version info
CURRENT_VERSION = "0.1.0"
GITHUB_REPO = "rzem-ai/rzem-ai-inference"
UPDATE_CHECK_INTERVAL = 3600  # Check every hour


class Version:
    """Semantic version comparison"""

    def __init__(self, version_str: str):
        # Remove 'v' prefix if present
        version_str = version_str.lstrip('v')
        parts = version_str.split('.')
        self.major = int(parts[0]) if len(parts) > 0 else 0
        self.minor = int(parts[1]) if len(parts) > 1 else 0
        self.patch = int(parts[2]) if len(parts) > 2 else 0

    def __gt__(self, other: 'Version') -> bool:
        if self.major != other.major:
            return self.major > other.major
        if self.minor != other.minor:
            return self.minor > other.minor
        return self.patch > other.patch

    def __str__(self) -> str:
        return f"{self.major}.{self.minor}.{self.patch}"


class AutoUpdater:
    """Automatic update manager"""

    def __init__(self, current_version: str = CURRENT_VERSION):
        self.current_version = Version(current_version)
        self.latest_version: Optional[Version] = None
        self.latest_release: Optional[Dict[str, Any]] = None
        self.update_available = False

        # Platform-specific executable names
        system = platform.system().lower()
        if system == "windows":
            self.executable_name = "RZEM-AI-Inference.exe"
        elif system == "darwin":
            self.executable_name = "RZEM-AI-Inference.app.zip"
        else:  # Linux
            self.executable_name = "RZEM-AI-Inference"

        # Update cache directory
        self.update_dir = Path.home() / ".rzem-ai-inference" / "updates"
        self.update_dir.mkdir(parents=True, exist_ok=True)

    async def check_for_updates(self) -> bool:
        """
        Check GitHub releases for updates.

        Returns:
            True if update is available, False otherwise
        """
        try:
            logger.info(f"Checking for updates (current: v{self.current_version})")

            async with aiohttp.ClientSession() as session:
                url = f"https://api.github.com/repos/{GITHUB_REPO}/releases/latest"
                headers = {"Accept": "application/vnd.github.v3+json"}

                async with session.get(url, headers=headers) as response:
                    if response.status != 200:
                        logger.warning(f"Failed to check for updates: HTTP {response.status}")
                        return False

                    data = await response.json()
                    self.latest_release = data

                    # Parse version from tag
                    tag_name = data.get("tag_name", "")
                    self.latest_version = Version(tag_name)

                    # Compare versions
                    if self.latest_version > self.current_version:
                        self.update_available = True
                        logger.info(f"Update available: v{self.latest_version}")
                        return True
                    else:
                        logger.info("No updates available")
                        return False

        except Exception as e:
            logger.error(f"Error checking for updates: {e}")
            return False

    def get_download_url(self) -> Optional[str]:
        """Get download URL for current platform"""
        if not self.latest_release:
            return None

        assets = self.latest_release.get("assets", [])
        for asset in assets:
            name = asset.get("name", "")
            if self.executable_name in name:
                return asset.get("browser_download_url")

        logger.warning(f"No asset found for platform: {self.executable_name}")
        return None

    async def download_update(self, progress_callback=None) -> Optional[Path]:
        """
        Download the update file.

        Args:
            progress_callback: Optional callback(bytes_downloaded, total_bytes)

        Returns:
            Path to downloaded file, or None if failed
        """
        download_url = self.get_download_url()
        if not download_url:
            logger.error("No download URL available")
            return None

        try:
            logger.info(f"Downloading update from: {download_url}")

            # Download to temp file
            output_file = self.update_dir / self.executable_name
            temp_file = output_file.with_suffix(output_file.suffix + ".tmp")

            async with aiohttp.ClientSession() as session:
                async with session.get(download_url) as response:
                    if response.status != 200:
                        logger.error(f"Download failed: HTTP {response.status}")
                        return None

                    total_size = int(response.headers.get("content-length", 0))
                    downloaded = 0

                    async with aiofiles.open(temp_file, "wb") as f:
                        async for chunk in response.content.iter_chunked(8192):
                            await f.write(chunk)
                            downloaded += len(chunk)

                            if progress_callback:
                                progress_callback(downloaded, total_size)

            # Verify download completed
            if temp_file.stat().st_size != total_size:
                logger.error("Download incomplete")
                temp_file.unlink()
                return None

            # Move to final location
            if output_file.exists():
                output_file.unlink()
            temp_file.rename(output_file)

            logger.info(f"Update downloaded: {output_file}")
            return output_file

        except Exception as e:
            logger.error(f"Error downloading update: {e}")
            return None

    async def verify_checksum(self, file_path: Path) -> bool:
        """
        Verify file checksum against release assets.

        Args:
            file_path: Path to downloaded file

        Returns:
            True if checksum matches, False otherwise
        """
        if not self.latest_release:
            logger.warning("No release info for checksum verification")
            return True  # Skip verification if no release info

        # Look for checksum in release body or assets
        checksums = {}
        for asset in self.latest_release.get("assets", []):
            name = asset.get("name", "")
            if "checksums" in name.lower() or "sha256" in name.lower():
                # Download and parse checksums file
                url = asset.get("browser_download_url")
                async with aiohttp.ClientSession() as session:
                    async with session.get(url) as response:
                        if response.status == 200:
                            text = await response.text()
                            for line in text.split("\n"):
                                if line.strip():
                                    parts = line.split()
                                    if len(parts) >= 2:
                                        checksums[parts[1]] = parts[0]

        if self.executable_name not in checksums:
            logger.warning("No checksum found for file, skipping verification")
            return True

        # Calculate file checksum
        sha256 = hashlib.sha256()
        async with aiofiles.open(file_path, "rb") as f:
            while chunk := await f.read(8192):
                sha256.update(chunk)

        calculated = sha256.hexdigest()
        expected = checksums[self.executable_name]

        if calculated != expected:
            logger.error(f"Checksum mismatch! Expected: {expected}, Got: {calculated}")
            return False

        logger.info("Checksum verified successfully")
        return True

    async def install_update(self, update_file: Path) -> bool:
        """
        Install the downloaded update.

        This replaces the current executable and restarts the application.

        Args:
            update_file: Path to downloaded update

        Returns:
            True if successful, False otherwise
        """
        try:
            logger.info("Installing update...")

            # Get current executable path
            if getattr(sys, 'frozen', False):
                # Running as PyInstaller bundle
                current_exe = Path(sys.executable)
            else:
                # Running as Python script (development mode)
                logger.warning("Cannot install update in development mode")
                return False

            # Backup current executable
            backup_file = current_exe.with_suffix(current_exe.suffix + ".backup")
            if backup_file.exists():
                backup_file.unlink()
            current_exe.rename(backup_file)

            # Install new executable
            import shutil
            shutil.copy2(update_file, current_exe)

            # Make executable (Unix)
            if platform.system() != "Windows":
                import os
                os.chmod(current_exe, 0o755)

            logger.info("Update installed successfully")

            # Restart application
            logger.info("Restarting application...")
            import subprocess
            subprocess.Popen([str(current_exe)] + sys.argv[1:])

            # Exit current process
            sys.exit(0)

        except Exception as e:
            logger.error(f"Error installing update: {e}")

            # Restore backup if installation failed
            if backup_file.exists() and not current_exe.exists():
                backup_file.rename(current_exe)

            return False

    async def download_and_install(self, progress_callback=None) -> bool:
        """
        Download and install update (convenience method).

        Args:
            progress_callback: Optional callback(bytes_downloaded, total_bytes)

        Returns:
            True if successful, False otherwise
        """
        # Download
        update_file = await self.download_update(progress_callback)
        if not update_file:
            return False

        # Verify
        if not await self.verify_checksum(update_file):
            logger.error("Update verification failed")
            update_file.unlink()
            return False

        # Install
        return await self.install_update(update_file)

    async def auto_check_loop(self):
        """Background task to periodically check for updates"""
        while True:
            try:
                await self.check_for_updates()
            except Exception as e:
                logger.error(f"Error in auto-check loop: {e}")

            # Wait before next check
            await asyncio.sleep(UPDATE_CHECK_INTERVAL)


# Singleton instance
_updater: Optional[AutoUpdater] = None


def get_updater() -> AutoUpdater:
    """Get global updater instance"""
    global _updater
    if _updater is None:
        _updater = AutoUpdater()
    return _updater
