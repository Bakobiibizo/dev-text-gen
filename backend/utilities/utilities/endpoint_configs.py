import os
from loguru import logger
from utilities.data_models import EndpointConfig, ConfigManager
from dotenv import load_dotenv

load_dotenv()


class EndpointConfigManager(ConfigManager):
    def __init__(self):
        super().__init__()
        self.set_configs()

        self.all_config = self.set_all_config()

    def set_configs(self):
        self.environment = str(os.getenv("environment"))
        self.version = str(os.getenv("version"))
        self.hostip = str(os.getenv("HOST_IP"))
        self.hub = self.get_config("hub")
        self.text = self.get_config("text")
        self.speech2text = self.get_config("speech2text")
        self.text2speech = self.get_config("text2speech")
        self.art = self.get_config("art")
        self.fingo = self.get_config("fingo")
        self.boardgame = self.get_config("boardgame")
        self.text2video = self.get_config("text2video")
        self.mistral = self.get_config("mistral")
        self.mixtral = self.get_config("mixtral")
        self.bakllava = self.get_config("bakllava")
        self.llava = self.get_config("llava")
        self.code = self.get_config("code")
        self.comfy = self.get_config("comfy")

    def set_all_config(self):
        self.all_config = {}
        for key, value in os.environ.items():
            self.all_config[key] = value
        return self.all_config

    def get_url(self, host, port, endpoint):
        return f"http://{host}:{port}{endpoint}"

    def get_config(self, value: str):
        # logger.debug(f"Getting {value} config")
        host = handle_error(os.getenv(f"{self.environment}_{value}_host"))
        port = handle_error(os.getenv(f"{self.environment}_{value}_port"))
        endpoint = handle_error(os.getenv(f"{self.environment}_{value}_endpoint"))

        config_map = {"host": host, "port": port, "endpoint": endpoint}

        config_map["url"] = self.get_url(**config_map)
        return EndpointConfig(**config_map)

    def set_item(self, key, value):
        logger.debug(f"Setting {key} to {value}")
        value = os.getenv(f"{self.environment}_{value}")
        self.__setattr__(key, value)


def handle_error(value):
    if value is None:
        raise ValueError(f"value cannot be None:\n{value}")
    else:
        return value


def getEndpointConfigManager():
    return EndpointConfigManager()


def main():
    return getEndpointConfigManager()


if __name__ == "__main__":
    manager = main()
    logger.debug(f"hub - {manager.hub}")
    logger.debug(f"text - {manager.text}")
    logger.debug(f"speech2text - {manager.speech2text}")
    logger.debug(f"text2speech - {manager.text2speech}")
    logger.debug(f"art - {manager.art}")
    logger.debug(f"video - {manager.text2video}")
    logger.debug(f"boardgame - {manager.boardgame}")
    logger.debug(f"fingo - {manager.fingo}")
    logger.debug(f"mistral - {manager.mistral}")
    logger.debug(f"mixtral - {manager.mixtral}")
    logger.debug(f"llava - {manager.llava}")
    logger.debug(f"bakllava - {manager.bakllava}")
    logger.debug(f"code - {manager.code}")
