Summary
===

I implemented a new pattern matching library, based on the insights from my bachelor thesis.
It can be used to match phrases, words and math formulae.
At the moment, the patterns are written as an XML file.
To test the new library, I implemented a new declaration spotter.

The PR also adds missing support for XML namespaces to the serialization code.

The Pattern Language
===

The patterns are written in an XML file (as in [this example](https://github.com/KWARC/llamapun/compare/master...jfschaefer:master#diff-0ac864542cb783e0c362b9ab17198fdd)).
A pattern file essentially contains a list of _rules_, which can reference each other.
I will try to provide an overview of how these rules look like. One day I might write a proper documentation.

Here is an example rule:
```xml
    <word_rule name="indefinite article">
        <meta>
            <description>
                Matches an indefinite article. Only covers lower case.
            </description>
        </meta>
        <word_or>
            <word>a</word>
            <word>an</word>
            <word>some</word>
            <word>any</word>
        </word_or>
    </word_rule>
```

This creates a rule for matching words. It has a name so that we can reference it later.
The `meta` node is optional and currently does not support much metadata.
Afterwards, we have the actual pattern that is matched by this rule. In this case, it is a `word_or` pattern, which matches a word, if any of the contained word patterns matches.

Here is a second word rule, referencing this rule:
```xml
    <word_rule name="article">
        <word_or>
            <word>the</word>
            <word>this</word>
            <word_ref ref="indefinite article" />
        </word_or>
    </word_rule>
```

There exist the following types of rules:
 * `mtext_rule` for matching the symbols in `math` nodes
 * `math_rule`  for matching `math` nodes (or parts of them)
 * `pos_rule` for matching part-of-speech (POS) tags
 * `word_rule` for matching words
 * `seq_rule` for matching sequence of words

Here is a more advanced example of two math rules that match an identifier using mutual recursion:
```xml
    <math_rule name="identifier">
        <math_or>
            <math_node name="mi">
                <mtext_ref ref="identifier symbol" />
            </math_node>
            <math_ref ref="indexed identifier" />
        </math_or>
    </math_rule>

    <math_rule name="indexed identifier">
        <math_or>
            <math_node name="msub">
                <math_children match_type="starts_with">
                    <math_ref ref="identifier" />
                </math_children>
            </math_node>
            <math_node name="msup">
                <math_children match_type="starts_with">
                    <math_ref ref="identifier" />
                </math_children>
            </math_node>
            <math_node name="msubsup">
                <math_children match_type="starts_with">
                    <math_ref ref="identifier" />
                </math_children>
            </math_node>
        </math_or>
    </math_rule>
```

For consistency, every pattern starts with a prefix, denoting what it matches. The only exception is the `phrase` pattern. It obviously matches sequences of words. Here is another example pattern that illustrates how the `phrase` pattern can be used and how patterns of different types can be combined:
```xml
    <phrase tag="NP">   <!-- noun phrase -->
        <match_type>shortest</match_type>
        <starts_with_seq containment="lessorequal">
            <seq_seq>
                <seq_word><word_ref ref="indefinite article"/></seq_word>
            </seq_seq>
        </starts_with_seq>
        <ends_with_seq>
            <seq_word>
                <word_math>
                    <math_or>
                        <math_ref ref="identifier" />
                        <math_ref ref="identifier sequence" />
                    </math_or>
                </word_math>
            </seq_word>
        </ends_with_seq>
    </phrase>
```

Markers
---
Now we can use these rules to find e.g. declarations in a document. However, we'd also be interested in identifying the components of this declaration (introduced identifier, restrictions, ...).
For this purpose, we can add markers to our patterns.
Here is a rule that matches and marks simple formulas that introduce and restrict identifiers like in $a \in M$ or $x \ge 0$:

```xml
    <math_rule name="single identifier restricted">
        <math_marker name="restriction" tags="math,introducing_identifier">
            <math_node name="mrow">
                <math_children match_type="exact">
                    <math_marker name="identifier">
                        <math_ref ref="identifier"/>
                    </math_marker>
                    <math_node name="mo">
                        <mtext_ref ref="relation" />
                    </math_node>
                    <math_or>
                        <math_node name="mrow" />
                        <math_ref ref="identifier" />
                    </math_or>
                </math_children>
            </math_node>
        </math_marker>
    </math_rule>
```

A marker has a name and optionally a list of tags associated with it. Markers can also be added to words and sequences of words. However, they are processed differently internally, as they correspond to ranges in the DNM, while math markers correspond to nodes in the DOM.

Currently, the only way to use the rules is by calling a `match_sentence` function, which takes a sentence and a seq_rule name and returns a list of all matches in that sentence.
A match is contains the matched markers as a tree structure.

Insights From The Example Declaration Spotter
===
Using [this pattern file](https://github.com/KWARC/llamapun/compare/master...jfschaefer:master#diff-0ac864542cb783e0c362b9ab17198fdd), I created a small example spotter to test the pattern matching library.
As KAT doesn't support string offsets yet, I simply exported the results into an HTML file ([attached as ZIP](https://github.com/KWARC/llamapun/files/769694/declarations.zip), because github didn't let me attach html). For simplicity, I ignored the tree structure of the resulting matches.

Insights:
* It is probably not a good idea to create one rule that does all the work (which is exactly what I did). You quickly end up with a lot of rules that do similar things, because the pattern language does not support things like "if we don't have a restricting noun phrase, require a restriction in the math node". I considered extending the language in different ways (e.g. adding rule templates). However, it might be better to have the logic in the rust code and keep the patterns simple.
* At least in this case, one of the main limitations is actually the precision of senna's syntactic parsing for our documents. We might be able to improve it using some refinements.
* There might still be some bugs in the pattern matching code - I found a few and it's not easy to test everything.
* In the spotter I created two parallel DNMs. One for the pattern matching (with a lot of normalization) and one for the output (no normalization, math nodes transformed into string representation of XML). Using the new serialization/deserialization it was easy to map the results. Maybe there are other interesting cases where we could use two DNMs in parallel.
* Creating the patterns from scratch was definitely a good idea and improved everything significantly. Maybe we should do it again soon :)

### Source:

This document was originally a [Pull Request description](https://github.com/KWARC/llamapun/pull/8)

