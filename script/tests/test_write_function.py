import unittest
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from .str_writer import StrWriter
from package_writer.write_function import write_function


def func1():
    pass

def func2() -> int:
    return 0

def func3(a,b,c):
    return a+b+c

def func4(a,b=1,c=1):
    return a+b+c

def func5(a: int,*args):
    return a+sum(args)

def func6(a,**kwargs):
    return a+sum(kwargs.values())

def func7(a,b,c=1,*args,**kwargs):
    return a+b+c+sum(args)+sum(kwargs.values())

def func8(a,*,b,c=1):
    return a+b+c

class TestWriteFunction(unittest.TestCase):
    def test_write_function(self):
        writer = StrWriter()
        writer.write("\n")
        write_function(writer, "func1", func1)
        write_function(writer, "func2", func2)
        write_function(writer, "func3", func3)
        write_function(writer, "func4", func4)
        write_function(writer, "func5", func5)
        write_function(writer, "func6", func6)
        write_function(writer, "func7", func7)
        write_function(writer, "func8", func8)

        expected = """
external dynamic func1();
external int func2();
external dynamic func3(dynamic a, dynamic b, dynamic c);
external dynamic func4(dynamic a, {dynamic b, dynamic c});
external Function func5;
external Function func6;
external Function func7;
external dynamic func8(dynamic a, {dynamic b, dynamic c});
"""
        self.assertEqual(writer.get_value(), expected)