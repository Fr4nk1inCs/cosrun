_EvaluatedNixValue = (
    None
    | bool
    | int
    | float
    | str
    | list[_EvaluatedNixValue]
    | dict[str, _EvaluatedNixValue]
)

def eval(path: str) -> _EvaluatedNixValue:
    """
    Evaluate a nix file and convert it to Python object.

    Args:
      - path (str): The path to the nix file.

    Returns:
      - _EvaluatedNixValue: The evaluated nix expression as any Python object

    Raises:
      - IOError: If the file cannot be read.
      - ParseError: If the nix file cannot be parsed.
      - EvaluationError: If the nix expression cannot be evaluated.
      - ConversionError: If the result cannot be converted to a Python object.

    Example:
    ```python
    # `path/to/file.nix` contains:
    # ```
    # {a = 1;}
    # ```
    >>> eval("path/to/file.nix")
    {'a': 1}
    ```
    """
    ...

def evals(expr: str, dir: str | None = None) -> _EvaluatedNixValue:
    """
    Evaluate a nix expression and convert it to Python object.

    Args:
      - expr (str): The nix expression to evaluate.
      - dir (str): The base directory to evaluate the expression in, we will
                   create a vitrual nix file as if the expr is in the file.

    Returns:
      - _EvaluatedNixValue: The evaluated nix expression as any Python object

    Raises:
      - ParseError: If the nix file cannot be parsed.
      - EvaluationError: If the nix expression cannot be evaluated.
      - ConversionError: If the result cannot be converted to a Python object.

    Example:
    ```python
    >>> evals("{a = 1;}")
    {'a': 1}
    ```
    """
    ...
