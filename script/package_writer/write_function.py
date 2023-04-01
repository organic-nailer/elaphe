import inspect
from typing import Optional
from .common import check_valid_identifier, get_annotation_to_type_str
from .write_document import write_document

def build_parameters(signature: inspect.signature, ignore_first_arg = False) -> Optional[str]:
    positional_only_list: list[inspect.Parameter] = []
    keyword_only_list: list[inspect.Parameter] = []
    normal_list: list[inspect.Parameter] = []
    var_keyword: Optional[inspect.Parameter] = None
    var_positional: Optional[inspect.Parameter] = None

    for param in signature.parameters.values():
        if param.kind == param.KEYWORD_ONLY:
            keyword_only_list.append(param)
        elif param.kind == param.POSITIONAL_ONLY:
            positional_only_list.append(param)
        elif param.kind == param.POSITIONAL_OR_KEYWORD:
            normal_list.append(param)
        elif param.kind == param.VAR_KEYWORD:
            var_keyword = param
        elif param.kind == param.VAR_POSITIONAL:
            var_positional = param
    
    if var_keyword is not None:
        # **kwargsはDartでは表現できないので全てを受け入れるFunctionにしてしまう
        return None
    if var_positional is not None:
        # *argsは対応が面倒なので全てを受け入れるFunctionにしてしまう
        return None
    
    if ignore_first_arg:
        normal_list = normal_list[1:]
    
    dart_param_list = []
    for param in normal_list:
        if not check_valid_identifier(param.name):
            continue
        if param.default is not inspect.Parameter.empty:
            # デフォルト値がある場合は、Dartではオプショナル引数になる
            keyword_only_list.append(param)
            continue

        if param.annotation:
            param_type = get_annotation_to_type_str(param.annotation)
        else:
            param_type = "dynamic"
        dart_param_list.append(f"{param_type} {param.name}")

    for param in positional_only_list:
        if not check_valid_identifier(param.name):
            continue
        if param.annotation:
            param_type = get_annotation_to_type_str(param.annotation)
        else:
            param_type = "dynamic"
        dart_param_list.append(f"{param_type} {param.name}")

    if len(keyword_only_list) > 0:
        keyword_param_list = []
        for param in keyword_only_list:
            if not check_valid_identifier(param.name):
                continue
            if param.annotation:
                param_type = get_annotation_to_type_str(param.annotation)
            else:
                param_type = "dynamic"
            if param_type != "dynamic":
                param_type = f"{param_type}?"
            keyword_param_list.append(f"{param_type} {param.name}")
        if len(keyword_param_list) > 0:
            dart_param_list.append("{" + ", ".join(keyword_param_list) + "}")

    return ", ".join(dart_param_list)

def write_function(f, func_name, func_obj, indent="", ignore_first_arg=False):
    if not check_valid_identifier(func_name):
        return
    
    write_document(f, func_obj, indent=indent)
    
    try:
        signature = inspect.signature(func_obj)
    except Exception:
        signature = None
    if not signature:
        f.write(f"{indent}external Function {func_name};\n")
        return
    
    if signature.return_annotation is inspect.Signature.empty:
        return_type = "dynamic"
    else:
        return_type = get_annotation_to_type_str(signature.return_annotation)

    dart_param_joined = build_parameters(signature, ignore_first_arg)

    if dart_param_joined is None:
        f.write(f"{indent}external Function {func_name};\n")
    else:
        f.write(f"{indent}external {return_type} {func_name}({dart_param_joined});\n")