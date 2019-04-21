# Setup:
# 1. Create the libllamapun.so dynamic library by running:
#   cargo build --release
# 2. Install gensim
#    pip install gensim
# And then execute this Python example from the top directory as:
#    python examples/vec2doc.py

from ctypes import cdll, c_char_p
from sys import platform

if platform == 'darwin':
    prefix = 'lib'
    ext = 'dylib'
elif platform == 'win32':
    prefix = ''
    ext = 'dll'
else:
    prefix = 'lib'
    ext = 'so'

lib = cdll.LoadLibrary('target/release/{}llamapun.{}'.format(prefix, ext))

tokenize_path = lib.word_tokenize_for_vec2doc
tokenize_path.restype = c_char_p
tokenize_path.argtypes = [c_char_p]

path = "tests/resources/0903.1000.html"
tokenized_doc = tokenize_path(path)
print("Tokenized example %s: " % path)
print(tokenized_doc)
