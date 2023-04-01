import inspect
from .common import dart_reserved_words, check_valid_identifier
from .write_variable import write_variable
from .write_function import write_function, build_parameters
from .write_document import write_document

def is_default_signature(signature) -> bool:
    return str(signature) == "(*args, **kwargs)"

def write_constructor(f, class_name, signature, indent=""):
    if not check_valid_identifier(class_name):
        return
    if signature is None:
        f.write(f"{indent}external {class_name}();\n")
        return
    parameters_str = build_parameters(signature, ignore_first_arg=True)
    if parameters_str is None:
        f.write(f"{indent}external {class_name}();\n")
        return
    f.write(f"{indent}external {class_name}({parameters_str});\n")

def write_class(f, class_name, class_obj):
    if not check_valid_identifier(class_name):
        return
    
    write_document(f, class_obj)

    # f.write(f"external dynamic {class_name}; /* {class_obj} */\n")
    f.write(f"class {class_name} {'{'}\n")
    member_list = inspect.getmembers(class_obj)

    # collect special attributes
    special_attributes = {}
    normal_member_list = []
    for member in member_list:
        if member[0].startswith("__") and member[0].endswith("__"):
            special_attributes[member[0]] = member[1]
        else:
            normal_member_list.append(member)

    if "__init__" in special_attributes or \
        "__new__" in special_attributes:
        signature = None
        try:
            # __new__が存在する場合はこちらのsignatureを使う
            if "__new__" in special_attributes:
                signature = inspect.signature(special_attributes["__new__"])
                if is_default_signature(signature):
                    signature = None
            
            # __new__が存在せず、__init__が存在する場合はこちらのsignatureを使う
            if signature is None and "__init__" in special_attributes:
                signature = inspect.signature(special_attributes["__init__"])
                if is_default_signature(signature):
                    signature = None
        except Exception:
            signature = None
        
        # Constructor
        write_constructor(f, class_name, signature, indent="  ")

    if "__getitem__" in special_attributes:
        # Index operator
        f.write("  external operator [](dynamic key);\n")
    if "__setitem__" in special_attributes:
        # Index operator
        f.write("  external operator []=(dynamic key, dynamic value);\n")

    for member in normal_member_list:
        if member[0] in dart_reserved_words:
            # print(f"Skipping {member[0]} because it is a reserved word in Dart")
            continue
        if inspect.isfunction(member[1]) or \
            callable(member[1]):
            write_function(f, member[0], member[1], indent="  ", ignore_first_arg=True)
            continue
        write_variable(f, member[0], member[1], indent="  ")
    f.write("}\n")
