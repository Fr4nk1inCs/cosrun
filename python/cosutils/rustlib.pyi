"""
A set of utilities for cosutils implemented in Rust.
"""

## src/nix.rs

evaluated = (
    None | bool | int | float | str | list[evaluated] | dict[str, evaluated]
)

class ParseError(ValueError):
    pass

class EvaluationError(ValueError):
    pass

class ConversionError(ValueError):
    pass

def eval(path: str) -> evaluated:
    """
    Evaluate a nix file and convert it to python object.

    Args:
      - path (str): The path to the nix file.

    Returns:
      - evaluated: The evaluated nix expression as any python object

    Raises:
      - ParseError: If the nix file cannot be parsed.
      - EvaluationError: If the nix expression cannot be evaluated.
      - ConversionError: If the result cannot be converted to a python object.

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

def evals(expr: str, dir: str | None = None) -> evaluated:
    """
    Evaluate a nix expression and convert it to python object.

    Args:
      - expr (str): The nix expression to evaluate.
      - dir (str): The base directory to evaluate the expression in, we will
                   create a vitrual nix file as if the expr is in the file.

    Returns:
      - evaluated: The evaluated nix expression as any python object

    Raises:
      - ParseError: If the nix file cannot be parsed.
      - EvaluationError: If the nix expression cannot be evaluated.
      - ConversionError: If the result cannot be converted to a python object.

    Example:
    ```python
    >>> evals("{a = 1;}")
    {'a': 1}
    ```
    """
    ...
