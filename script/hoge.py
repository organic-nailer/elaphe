from package_writer import write_function
import pandas as pd

class Writer:
    def __init__(self):
        self.txt = ""

    def write(self, text):
        self.txt += text

w = Writer()
write_function.write_function(w, "read_csv", pd.read_csv)

# print(w.txt)
