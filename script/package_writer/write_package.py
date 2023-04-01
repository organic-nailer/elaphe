import os
import sys
import inspect
import pydoc
import pkgutil

from importlib import import_module
from importlib.util import find_spec
from types import ModuleType
from .common import dart_reserved_words
from .write_class import write_class
from .write_function import write_function
from .write_variable import write_variable

def write_package(lib: ModuleType, name: str, root_dir: str) -> None:
    print(f"Writing {name} to {root_dir}")
    filename = "/".join(name.split(".")) + ".d.dart"
    out_file = os.path.join(root_dir, filename)
    member_list = inspect.getmembers(lib)
    os.makedirs(os.path.dirname(out_file), exist_ok=True)
    if os.path.isfile(out_file):
        os.remove(out_file)
    with open(out_file, "w", encoding="utf-8_sig") as f:
        for member in member_list:
            if member[0] in dart_reserved_words:
                # print(f"Skipping {member[0]} because it is a reserved word in Dart")
                continue
            if inspect.ismodule(member[1]):
                # モジュールは参照なのでここで処理する必要なし
                continue
                # if member[0] == "os":
                #     continue
                # if member[0] != member[1].__name__.split(".")[-1]:
                #     continue
                # if member[0] in dir.split("/"):
                #     # Avoid infinite recursion
                #     continue
                # write_module(member[1], dir + module_name + "/")
            elif inspect.isfunction(member[1]) or \
               inspect.isbuiltin(member[1]):
                write_function(f, member[0], member[1])
            elif inspect.isclass(member[1]):
                write_class(f, member[0], member[1])
            elif callable(member[1]):
                write_function(f, member[0], member[1])
            else:
                write_variable(f, member[0], member[1])

def check_ignore_module(modname: str, only_package = True) -> bool:
    if modname.find("test") != -1:
        # Ignore test modules
        return True
    if modname.find("._") != -1:
        # Ignore private modules
        return True
    if only_package:
        mod_spec = find_spec(modname)
        if mod_spec is None or mod_spec.submodule_search_locations is None:
            # Ignore modules that are not in a directory
            # because there may be a program
            return True
    return False

def write_package_recursively(lib: ModuleType, dir: str, only_package = True) -> None:
    """
    指定されたモジュールとそのサブモジュールを再帰的に処理する
    """
    write_package(lib, lib.__name__, dir)
    def onerror(modname):
        print(f"Error: cannot process module {modname}.", file=sys.stderr)
    for importer, modname, ispkg in pkgutil.walk_packages(path=lib.__path__, prefix=lib.__name__ + ".", onerror=onerror):
        if not check_ignore_module(modname, only_package):
            try:
                write_package(import_module(modname), modname, dir)
            except Exception as e:
                print(f"Error: cannot process module {modname}.", file=sys.stderr)
                print(e, file=sys.stderr)

    # def visitor(path, modname, desc):
    #     if modname == lib.__name__ or \
    #         modname.startswith(lib.__name__ + "."):
    #         if not check_ignore_module(modname):
    #             try:
    #                 write_package(import_module(modname), modname, dir)
    #             except Exception as e:
    #                 print(f"Error: cannot process module {modname}.", file=sys.stderr)
    # def onerror(modname):
    #     print(f"Error: cannot process module {modname}.", file=sys.stderr)
    # pydoc.ModuleScanner().run(visitor, onerror=onerror)
