# Setup:
# 1. Create the libllamapun.so dynamic library by running:
#   cargo build --release
# 2. Install gensim
#    pip install gensim
# And then execute this Python example from the top directory as:
#    python3 examples/gensim_doc2vec.py

from ctypes import cdll, c_char_p
import os
import numpy
import re
import sys
import multiprocessing


from gensim.models.doc2vec import Doc2Vec, TaggedDocument, FAST_VERSION
from gensim.test.utils import get_tmpfile

cores = multiprocessing.cpu_count()
assert FAST_VERSION > -1, "This will be painfully slow otherwise"

# llamapun-loading code


if sys.platform == 'darwin':
    prefix = 'lib'
    ext = 'dylib'
elif sys.platform == 'win32':
    prefix = ''
    ext = 'dll'
else:
    prefix = 'lib'
    ext = 'so'

lib = cdll.LoadLibrary('target/release/{}llamapun.{}'.format(prefix, ext))

tokenize_path = lib.word_tokenize_for_vec2doc
tokenize_path.restype = c_char_p
tokenize_path.argtypes = [c_char_p]

#### main experiment example ####

# Create iterator for Doc2Vec vocabulary
# Thanks to Philipp Scharpf for initial code example!


class LlamapunTokenizedDocumentIterator(object):
    def __init__(self, path_list, labels_list):
        self.labels_list = labels_list
        self.path_list = path_list
        # Caching is a possibility for small datasets.
        # In the case of arXiv, we pay a price of 1 GB of allocated RAM for 1000 documents
        # and hence caching as a strategy has to be done differently.
        #
        # a separate idea is to cache locally on disk,
        # but once we concede to using the space needed for word-tokenizing the entire arXiv, we
        # might as well pre-generate that resource from llamapun and sidestep the runtime in this example
        # since the very purpose of this specific file is to demonstrate the Rust-Python connection,
        # we stick to using a cached traversal in RAM, and suggest to readers to explore
        # more performance-friendly strategies for large resources.
        # another idea would be to integrate the original doc2vec of Mikolov via C,
        # without the python layer entirely
        self.cached_doc = []

    def __iter__(self):
        for idx, path in enumerate(self.path_list):
            try:
                cached = self.cached_doc[idx]
                yield cached
            except IndexError:
                if idx > 0 and idx % 1000 == 0:
                    print("tokenizing document %d of corpus" % idx)
                tokenized = tokenize_path(path.encode('utf-8'))
                words = tokenized.decode('utf-8').split()
                item = TaggedDocument(words,
                                      [self.labels_list[idx]])
                self.cached_doc.append(item)
                yield item


def docs2vec(paths, labels):
    # Helpful link with advanced performance tuning use of doc2vec:
    # https: // github.com/RaRe-Technologies/gensim/blob/develop/docs/notebooks/doc2vec-IMDB.ipynb

    # Build Doc2Vec model
    iterator = LlamapunTokenizedDocumentIterator(paths, labels)
    model = Doc2Vec(
        vector_size=300, window=10, min_count=5, workers=cores, alpha=0.025, min_alpha=0.025)
    print("building vocabulary")
    model.build_vocab(iterator)
    print("%s vocabulary scanned & state initialized" % model)

    # Train Doc2Vec model
    print("training model")
    model.train(documents=iterator, epochs=10,
                total_examples=model.corpus_count)
    model.alpha -= 0.002
    model.min_alpha = model.alpha
    model.total_examples = model.corpus_count

    return model

### main execution loop ###


# Set corpus path
corpus_path = "tests/resources"
argcount = len(sys.argv[1:])
if argcount > 0:
    corpus_path = sys.argv[1]

# Collect documents, and their labels
paths = []
labels = []

# Fetch content and labels of documents
for directory, subdirList, fileList in os.walk(corpus_path):
    for filename in fileList:
        if filename.endswith(".html"):
            # store text data of document
            paths.append(directory + "/" + filename)
            label = filename
            m = re.match("^([a-z][^\d]+)", filename)
            if m:
                # old arxiv id, grab everything until first digit
                label = m[0]
            else:
                m = re.match("^([^.])+", filename)
                if m:
                    # new arxiv, category is all before first dot (but is temporal, not topical)
                    # only for example purposes
                    label = m[0]
            labels.append(label)

# Limit to 50,000 for the cached approach, on a 64 GB RAM machine
paths = paths[0:50000]
labels = labels[0:50000]

# Build Doc2Vec text model
print("starting docs2vec on %d total paths" % len(paths))
model = docs2vec(paths, labels)
model.save("my_llamapun_powered_doc2vec_model")

# -- scratch --
# With caching the tokenized documents in RAM:
# 64 GB are exhausted after 79,000 cached documents (as word-tokenized paragraphs)
#
# tokenizing document 78000 of corpus
# tokenizing document 79000 of corpus
# Killed
#
# real	261m26.614s
# user	230m23.962s
# sys	14m24.250s

# With caching, capped at 50,000 documents
# training takes multi-threading with a stable 2900% load (32 threads provided, threadripper 1950x)
#
# Time:
# real	198m40.172s
# user	1289m17.262s
# sys	9m26.623s
