class StrWriter:
    def __init__(self):
        self._str = ""

    def write(self, s):
        self._str += s

    def get_value(self):
        return self._str