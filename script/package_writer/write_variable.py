from .common import value_to_type_str, check_valid_identifier
from .write_document import write_document

def write_variable(f, var_name, var_obj, indent=""):
    if not check_valid_identifier(var_name):
        return
    write_document(f, var_obj, indent=indent)
    var_type = value_to_type_str(var_obj)
    f.write(f"{indent}external {var_type} {var_name};\n")