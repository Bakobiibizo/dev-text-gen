import openai  # openai v1.0.0+
from litellm.llms.ollama import (
    ollama_aembeddings,
    ollama_completion_stream,
    OllamaConfig,
)


def openai_client(prompt="write a short poem about a duck wearing a fedora"):
    client = openai.OpenAI(
        api_key="API Key: sk-Rer6_qGQkuFhanO65T5B7g", base_url="http://0.0.0.0:38095"
    )  # set proxy to base_url
    # request sent to model set on litellm proxy, `litellm --model`
    response = client.chat.completions.create(
        model="ollama/dolphin-mistral",
        messages=[{"role": "user", "content": prompt}],
    )

    print(response)


if __name__ == "__main__":
    openai_client()


from litellm import acompletion
