from typing import Any


dart_reserved_words = [
    "else", "assert", "break", "case", "catch", "class", "const", "continue", "default", "do", "enum", "extends", "false", "final", "finally", "for", "if", "in", "is", "new", "null", "return", "super", "switch", "this", "true", "try", "var", "void", "while", "with",
    "double", "int", "String",
    "abstract", "as", "covariant", "deferred", "dynamic", "export", "external", "factory", "Function", "get", "import", "interface", "library", "mixin", "operator", "part", "set", "static", "typedef",
    "bool", "Enum", "List", "Map", "num", "Object", 
    "main"
]

def value_to_type_str(value: Any) -> str:
    if isinstance(value, int):
        return "int"
    elif isinstance(value, float):
        return "double"
    elif isinstance(value, str):
        return "String"
    elif isinstance(value, bool):
        return "bool"
    elif isinstance(value, list):
        return "List<dynamic>"
    elif isinstance(value, dict):
        return "Map<dynamic, dynamic>"
    else:
        return "dynamic"

def get_annotation_to_type_str(var_obj: Any) -> str:
    if var_obj is int or var_obj == "int":
        return "int"
    elif var_obj is float or var_obj == "float":
        return "double"
    elif var_obj is str or var_obj == "str":
        return "String"
    elif var_obj is bool or var_obj == "bool":
        return "bool"
    elif var_obj is list or var_obj == "list":
        return "List<dynamic>"
    elif var_obj is dict or var_obj == "dict":
        return "Map<dynamic, dynamic>"
    else:
        return "dynamic"
    
def check_valid_identifier(identifier: str) -> bool:
    if identifier in dart_reserved_words:
        return False
    if identifier.startswith("_"):
        return False
    return True