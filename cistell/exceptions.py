class ConfigError(Exception):
    """Base class for all the config related errors"""


class ConfigMultiInheritanceError(ConfigError):
    """Error related with multiinheritance of config fields"""
