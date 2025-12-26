from pydantic import BaseModel
from abc import ABC, abstractmethod
from typing import Any, Union, Optional, List
from typing import Optional, List, Union
from pathlib import Path


class EndpointConfig(BaseModel):
    host: str
    port: str
    endpoint: str
    url: str


class ConfigManager(ABC):
    def __init__(self):
        self.environment: str
        self.version: str
        self.hub: EndpointConfig
        self.text: EndpointConfig
        self.speech2text: EndpointConfig
        self.text2speech: EndpointConfig
        self.art: EndpointConfig
        self.boardgame: EndpointConfig
        self.text2video: EndpointConfig
        self.mistral: EndpointConfig
        self.mixtral: EndpointConfig
        self.llava: EndpointConfig
        self.bakllava: EndpointConfig
        self.code: EndpointConfig
        self.comfy: EndpointConfig

    @abstractmethod
    def get_config(self, value: str):
        pass

    @abstractmethod
    def get_url(self, host, port, version, endpoint) -> str:
        pass

    @abstractmethod
    def set_item(self, key, value) -> None:
        pass


class OAIMessage(BaseModel):
    role: str
    content: str


class TokenUsage(BaseModel):
    total: int
    prompt: int
    request: int
    response: int


class ChoicesMessage(BaseModel):
    finish_reason: str
    index: int
    logprobs: str
    finish_reason: str
    Messages: List[OAIMessage]


class OAIResponse(BaseModel):
    id: Optional[Union[int, None]]
    object: Optional[Union[str, None]]
    created: Optional[Union[int, None]]
    model: Optional[Union[str, None]]
    system_fingerprint: Optional[Union[str, None]]
    choices: Optional[Union[ChoicesMessage, None]]
    usage: Optional[Union[TokenUsage, None]]


class BoardGameRequest(BaseModel):
    imageId: Optional[str] = None
    stylePositive: Optional[str] = None
    styleNegative: Optional[str] = None
    styleInput: Optional[str] = None
    inputSeed: Optional[int] = None
    boardControl: Optional[float] = None
    inputBoard: Optional[str] = None
    numberOfSteps: Optional[int] = None
    creativity: Optional[float] = None


def getPrompt(
    imageId,
    positivePrompt,
    negativePrompt,
    stylePositive,
    styleNegative,
    inputSeed,
    boardControl,
    inputBoard,
    numberOfSteps,
    creativity,
):
    file_path = Path.cwd() / "src" / "static" / "prompt.json"

    prompt_text = file_path.read_text()

    prompt_map = {
        "{{board_strength}}": str(boardControl),
        "{{board_image}}": str(inputBoard),
        "{{seed}}": str(inputSeed),
        "{{steps}}": str(numberOfSteps),
        "{{creativity}}": str(creativity),
        "{{positive_prompt}}": str(positivePrompt),
        "{{negative_prompt}}": str(negativePrompt),
        "{{positive_style}}": str(styleNegative),
        "{{negative_style}}": str(stylePositive),
        "{{image_id}}": str(imageId),
    }

    for key, value in prompt_map.items():
        prompt_text = prompt_text.replace(key, str(value))
    return prompt_text


class OllamaRequest(BaseModel):
    images: Optional[List[str]] = None
    prompt: Optional[str] = None
    model: Optional[str] = None


class TextRequest(BaseModel):
    prompt: Optional[str] = None
    model: Optional[str] = None


class ArtGenerationRequest(BaseModel):
    prompt: Optional[Any] = None


class Text2VideoRequest(BaseModel):
    prompt: Optional[str] = None
    fps: Optional[int] = 129


class Image2VideoRequest(BaseModel):
    image: Optional[str] = None
    prompt: Optional[str] = None
    fps: Optional[int] = 129


class Speech2TextResponse(BaseModel):
    audio: str
    audio_filename: Optional[str] = None


class Text2SpeechRequest(BaseModel):
    prompt: str


class BoardgameArtRequest(BaseModel):
    themeData: int
    board: Optional[int] = None
    customBoard: Optional[str] = None
    avatar1: Optional[str] = None
    avatar2: Optional[str] = None
    avatar3: Optional[str] = None
    avatar4: Optional[str] = None
    imprintValue: int
    creativity: float
    steps: int
    prompt: str
    randomize: bool
