"""Shared protocol types for client/server/local modes"""

from enum import Enum
from typing import Optional
from pydantic import BaseModel


class RuntimeMode(str, Enum):
    LOCAL = "local"
    SERVER = "server"
    CLIENT = "client"


class RuntimeConfig(BaseModel):
    """Configuration for runtime operation mode"""
    mode: RuntimeMode
    server_url: Optional[str] = None
    server_port: Optional[int] = None
    dev_mode: bool = False

    @classmethod
    def local(cls, dev_mode: bool = False) -> "RuntimeConfig":
        return cls(mode=RuntimeMode.LOCAL, dev_mode=dev_mode)

    @classmethod
    def server(cls, port: int) -> "RuntimeConfig":
        return cls(mode=RuntimeMode.SERVER, server_port=port)

    @classmethod
    def client(cls, server_url: str) -> "RuntimeConfig":
        return cls(mode=RuntimeMode.CLIENT, server_url=server_url)
