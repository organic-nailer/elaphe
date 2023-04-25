from importlib import import_module
import sys
import os
from package_writer import write_package_recursively

if __name__ == "__main__":
    argv = sys.argv
    if len(argv) < 2:
        print("Usage: python ./script/gen_type_stubs.py <module_name> [--only-package]")
        exit(1)

    # Add current directory to sys.path
    sys.path.append(os.getcwd())
    
    only_package = "--only-package" in argv
    module_name = argv[1]
    target_mod = import_module(module_name)
    write_package_recursively(target_mod, "./elaphe/", only_package)
