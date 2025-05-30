_JsonValue = (
    None
    | bool
    | int
    | float
    | str
    | list["_JsonValue"]
    | dict[str, "_JsonValue"]
)

def load(path: str) -> _JsonValue:
    """
    Parse a JSONC (JSON with comments) file and convert it to a Python object.

    Args:
      - path (str): The path to the JSONC file.

    Returns:
      - _JsonValue: A Python object representing a valid JSON value.

    Raises:
      - IOError: If the file cannot be read.
      - ParseError: If the content is not valid JSONC.
    """
    pass

def loads(expr: str) -> _JsonValue:
    """
    Parse a JSONC (JSON with comments) string and convert it to a Python object.

    Args:
      - content (str): The JSONC content as a string.

    Returns:
      - _JsonValue: A Python object representing a valid JSON value.

    Raises:
      - ParseError: If the content is not valid JSONC.
    """
    pass
