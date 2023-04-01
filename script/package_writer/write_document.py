def write_document(f, obj, indent=""):
    # write document
    if obj.__doc__:
        doc = obj.__doc__
        doc = f"\n{indent}/// " + doc.replace("\n", f"\n{indent}/// ")
        f.write(doc + "\n")