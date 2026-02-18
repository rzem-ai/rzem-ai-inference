"""Chat API mixin — Claude AI assistant integration."""

from __future__ import annotations

import logging
import uuid
from typing import Any

from backend.db.database import Database
from backend.services.chat_service import ChatService

logger = logging.getLogger(__name__)


class ChatAPI:
    """pywebview js_api mixin for chat functionality."""

    def __init__(self, chat_service: ChatService, db: Database) -> None:
        self._chat = chat_service
        self._chat_db = db

    def chat_is_configured(self, **kwargs) -> dict[str, Any]:
        try:
            return {"status": "success", "configured": self._chat.is_configured}
        except Exception as e:
            logger.error("Failed to check chat config: %s", e)
            return {"status": "error", "message": str(e)}

    def chat_set_api_key(self, api_key: str, **kwargs) -> dict[str, Any]:
        try:
            self._chat.set_api_key(api_key)
            self._chat_db.set_setting("CLAUDE_API_KEY", api_key)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to set API key: %s", e)
            return {"status": "error", "message": str(e)}

    def chat_create_conversation(self, title: str = "New Chat", **kwargs) -> dict[str, Any]:
        try:
            conv_id = str(uuid.uuid4())
            conversation = self._chat_db.create_conversation(conv_id, title)
            return {"status": "success", "conversation": conversation}
        except Exception as e:
            logger.error("Failed to create conversation: %s", e)
            return {"status": "error", "message": str(e)}

    def chat_get_conversations(self, **kwargs) -> dict[str, Any]:
        try:
            conversations = self._chat_db.get_conversations()
            return {"status": "success", "conversations": conversations}
        except Exception as e:
            logger.error("Failed to get conversations: %s", e)
            return {"status": "error", "message": str(e)}

    def chat_get_messages(self, conversation_id: str, **kwargs) -> dict[str, Any]:
        try:
            messages = self._chat_db.get_conversation_messages(conversation_id)
            return {"status": "success", "messages": messages}
        except Exception as e:
            logger.error("Failed to get messages: %s", e)
            return {"status": "error", "message": str(e)}

    def chat_delete_conversation(self, conversation_id: str, **kwargs) -> dict[str, Any]:
        try:
            self._chat_db.delete_conversation(conversation_id)
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to delete conversation: %s", e)
            return {"status": "error", "message": str(e)}

    def chat_send_message(
        self,
        conversation_id: str,
        content: str,
        image_paths: list[str] | None = None,
        generation_context: dict[str, Any] | None = None,
        display_text: str | None = None,
        **kwargs,
    ) -> dict[str, Any]:
        try:
            self._chat.send_message(
                conversation_id=conversation_id,
                user_content=content,
                image_paths=image_paths,
                generation_context=generation_context,
                display_text=display_text,
            )
            return {"status": "success"}
        except Exception as e:
            logger.error("Failed to send message: %s", e)
            return {"status": "error", "message": str(e)}

    def poll_chat_events(self, **kwargs) -> dict[str, Any]:
        try:
            events = self._chat.drain_events()
            return {"status": "success", "events": events}
        except Exception as e:
            logger.error("Failed to poll chat events: %s", e)
            return {"status": "error", "message": str(e)}
