# Setup:
# 1. Create the libllamapun.so dynamic library by running:
#   cargo build --release
# 2. Install gensim
#    pip install gensim
# And then execute this Python example from the top directory as:
#    python examples/vec2doc.py

import os
import numpy
from gensim.models.doc2vec import Doc2Vec, TaggedDocument
from gensim.test.utils import get_tmpfile
# llamapun-loading code
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

#### main experiment example ####

# Create iterator for Doc2Vec vocabulary
# Thanks to Philipp Scharpf for initial code example!


class LlamapunTokenizedDocumentIterator(object):
    def __init__(self, path_list, labels_list):
        self.labels_list = labels_list
        self.path_list = path_list

    def __iter__(self):
        for idx, path in enumerate(self.path_list):
            tokenized = tokenize_path(path.encode('utf-8'))
            words = tokenized.decode('utf-8').split()
            yield TaggedDocument(words,
                                 [self.labels_list[idx]])


def docs2vec(paths, labels):
    # Build Doc2Vec model
    iterator = LlamapunTokenizedDocumentIterator(paths, labels)
    model = Doc2Vec(
        vector_size=300, window=10, min_count=5, workers=16, alpha=0.025, min_alpha=0.025, epochs=20)
    print("building vocabulary")
    model.build_vocab(iterator)

    # Train Doc2Vec model
    for epoch in range(2):
        print('iteration ' + str(epoch+1))
        model.train(iterator, epochs=model.iter,
                    total_examples=model.corpus_count)
        model.alpha -= 0.002
        model.min_alpha = model.alpha
        model.total_examples = model.corpus_count

    # Generate Doc2Vec vectors
    docVecs = []
    for label in labels:
        docVecs.append(model.docvecs[label])

    return model, docVecs

### main execution loop ###


# Set file path
datasetPath = "tests/resources"
# Collect documents, and their labels
paths = []
labels = []

# Fetch content and labels of documents
for directory, subdirList, fileList in os.walk(datasetPath):
    for filename in fileList:
        if filename.endswith(".html"):
            # store text data of document
            paths.append(directory + "/" + filename)
            # store label of document (just the name here, it's an example)
            labels.append(filename)

# Build Doc2Vec text model
model, vectors = docs2vec(paths, labels)

model.save("my_llamapun_powered_doc2vec_model")
for (idx, vector) in enumerate(vectors):
    numpy.savetxt("doc2vec_"+labels[idx]+"_vector.txt", vector)
