import tiktoken
from typing import List
from utilities.data_models import TokenUsage


class TokenManager(TokenUsage):
    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)
        self.session_total = 0
        self.request_tokens = 0
        self.response_tokens = 0
        self.total_tokens = kwargs["total"]
        self.completion_tokens = kwargs["response_tokens"]
        self.prompt_tokens = kwargs["prompt_tokens"]
        self.historical_list: List[TokenUsage] = []

    def remove(self, index) -> str:
        self.historical_list.pop(index)
        if index == 0:
            self.session_total = 0
            self.prompt_tokens = 0
            self.request_tokens = 0
            self.response_tokens = 0
            self.historical_list = 0

        return "index removed"

    def update(self, total: int, request: int, response: int) -> str:
        self.session_total += total
        self.prompt_tokens = total
        self.request_tokens = request
        self.response_tokens = response

    def create_embedding(self, text: str, encoding_name: str = "cl100k_base"):
        """Returns the number of tokens in a text string."""
        encoding = tiktoken.get_encoding(encoding_name)
        return encoding.encode(text)

    def count_tokens(self, string: str, encoding_name: str = "cl100k_base") -> int:
        """Returns the number of tokens in a text string."""
        encoding = tiktoken.get_encoding(encoding_name)
        return len(encoding.encode(string))
