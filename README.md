# Bebop Lang
## LISP based Markdown preprocessor
Bebop is so named to allow people to freely express themselves. We want to let people write better documents through simple bebop programs. The basis of the language is Markdown, a simplified version of HTML. Writers can quickly pick up the syntax and draft very sophisticated documents from the start. We inject another layer on top of the Markdown with a simple lisp dialect that preprocesses the Markdown document before rendering. This should look familiar to PHP for those in the know. You can use the lisp to define values and functions that can be re-used all throughout the program. In turn, one can write documents that share elements and structure to build a custom framework.
A bebop program looks like a regular markdown document that contains lisp code. All bebop programs output a string of text that represents the document. Programs can use entirely markdown or entirely lisp if the author chooses. The runtime converts all markdown into special lisp functions, so one can imagine the markdown as syntactic sugar for code. After converting all the markdown into lisp, the resulting code will look like a series of statements that must resolve to a string. Finally, the runtime executes the lisp and concatenates each element together to produce the resulting document. We will now define the specs for both the markdown and the LISP side of things.
### Markdown
Get started by reading the [markdown guide](https://www.markdownguide.org/getting-started/); it touches upon most of the syntax in this language. We use markdown to specify the content in our documents. It is analogous to HTML, with which the reader may be familiar. 
#### Headings
A heading translates to an `h1` or differently numbered header in HTML. 
To specify an `h1`, use the `#` symbol followed by a space and then text. The number of `#` symbols determines the header level.
Example:
```md
# Top level H1
## Second level H2
### Third level H3
#### Fourth level H4
##### Fifth level H5
###### Sixth level H6
```
#### Lists
Lists can be either ordered or unordered, both of which correspond to `ol` and `ul` in HTML respectively. We also provide a task list as well which is not found in HTML. 
An ordered list is a series of lines that begin with *any number*, a `.`, and a space followed by text.
```md
1. the numbers
3. do not matter
1. and will order themselves
4000. correctly when rendered
```
An unordered list is a series of lines that begin with a `-` and a space followed by text.
```md
- items in
- a ul
- don't lend
- themselves to
- any particular
- order
```
A task list is a series of lines that begin with `- [ ]` and a space followed by text. To check the box as done, use `- [x]`.
```md
- [ ] cookies
- [x] toilet paper
- [ ] beans
- [ ] milk
```
#### Codeblock
A codeblock does not correspond directly to an HTML element, although they are very commonly found on the internet. The idea is that the formatting in the codeblock is preserved when rendering.
Two write a codeblock, use 3 backticks followed by the language of code being used, then any series of lines of text, and then a line with 3 backticks.
Example:
*example goes here*

#### Blockquote
A blockquote corresponds directly to the `blockquote` tag in HTML. It is written by using a `>` followed by a space and then text. It ends when the line ends.
Example:
```md
> Four score and seven years ago...
```
#### Horizontal Rule
A horizontal corresponds to the `hr` tag in HTML. It is written by simply using `---` on a line.
Example:
```md
---
```
#### Inline Elements
If none of the above elements are used, then a line with text in it is considered a paragraph. There can be inline elements inside paragraphs.

##### Bold, Italic, Inline Code & Strikethrough
These elements correspond to their similarly named HTML tags.
Example:
```md
I think *italic text looks cool*, **bold text looks like yelling**, and ~~struck through text looks redacted~~.
```

##### Link & Image
Both links and images have corresponding tags in HTML. We also provide an external link that will open a new tab when clicked on.
Example:
```md
[Link text](/bebop)
^[External link](https://google.com)
![Image alt text](https://picsum.photos/200)
```

##### Color Swatch
This is a custom element, that represents a given hex code.
Example:
```md
#ff5523 is a cool color yeah?
```

##### Plaintext
Any text that does not fall under the other inline elements' syntax.
Example:
```md
Plain planes find themselves on the plain.
```

### LISP
#### Grammar
```g
Sexp = ( Symbol+ )
Qexp = [ Symbol+ ]
Number = 1234567890
Symbol = _+\\:-*/=|!&%a-zA-Z1234567890
String = “Symbol”
```

#### Syntax & Types
Our types are simple and cover pretty much all of our bases. I don't believe that we will need to change our design here. 

##### Number
Numbers like we are all familiar with. (ie. `1`, `1.1`, `1.1e+13`, `1.1e-13`)
##### Symbol
Symbols are names that can be assigned to any value. (ie. `add`, `def`, `fun`, `some-var`)
Usage: `def [symbol-name] value`
##### String
Strings are characters delimited by double quotes. (ie. `'c'ect ci nest pa un pipe?\'`, `\'hg king\'`)
##### S-Expression
S-Expressions are used to call and evaluate functions. (ie. `(+ 1 2 3)`, `(- (+ 9 1) (* 5 2))`, `(list 1 2 3 4)`, `(== [] [])`)
Usage: `(function arg0 arg1 arg2)`
##### Q-Expression
Q-Expressions are lists of values, remains unevaluated. (ie. `[1 1 1 1]`, `[+ 9 (== [] [])]`)
Usage: `[elem0 elem1 elem2]`
##### Lambda
Lambda functions are how you build functions, can be partially applied. (ie. `(\ [a b] [+ a b])`)
Usage: `(\ [arg-list] [body])`