use bebop_lang::lisp::{Compile, Lisp};

fn main() {
    let x = r#"|concat

(def [fun]
    (\ [args body] 
        [def (list (head args)) 
        (\ (tail args) body)]))

(fun [h1 children]
    [concat "<h1>" children "</h1>"])

(fun [h2 children]
    [concat "<h2>" children "</h2>"])

(fun [h3 children]
    [concat "<h3>" children "</h3>"])

(fun [h4 children]
    [concat "<h4>" children "</h4>"])

(fun [h5 children]
    [concat "<h5>" children "</h5>"])

(fun [h6 children]
    [concat "<h6>" children "</h6>"])

(fun [code children]
    [concat "<code>" children "</code>"])

(fun [pre children]
    [concat "<pre>" children "</pre>"])

(fun [p children]
    [concat "<p>" children "</p>"])

(fun [i children]
    [concat "<i>" children "</i>"]) 

(fun [b children]
    [concat "<b>" children "</b>"])

(fun [li children]
    [concat "<li>" children "</li>"])

(fun [ul children]
    [concat "<ul>" children "</ul>"])

(fun [ol children]
    [concat "<ol>" children "</ol>"])

(fun [img src alt]
    [concat "<img src='" src "' alt='" alt "' />"])
    
(fun [a href children]
    [concat "<a href='" href "'>" children "</a>"])

(def [hr]
    "<hr/>")

(def [true]
    1)
    
(def [false]
    0)

(def [nil] ())

(fun [not n]
    [if (== n 0) [1] [0]])

(fun [is-nil n] 
    [== n nil])

(fun [not-nil n] 
    [not (== n nil)])

(fun [dec n] [- n 1])

(def [fun] 
    (\ [args body] 
        [def (list (head args)) 
        (\ (tail args) body)]))

(fun [cons x xs]
    [join
        (if (== x [])
            [x]
            [list x])
        xs])

(fun [empty l] 
    [if (== l []) 
        [true] 
        [false]])

(fun [len l] 
    [if (empty l) 
        [0] 
        [+ 1 (len (tail l))]])

(fun [rec target base step]
    [if (== 0 target)
        [base]
        [step (dec target)
            (\ [] [rec (dec target) base step])]])

(fun [rec-list target base step]
    [if (== 0 (len target))
        [base]
        [step 
            (head target)
            (\ [] [rec-list (tail target) base step])]])

(fun [map target mapper]
    [rec-list target [] (\ [e es] [cons (mapper e) (es)])])

(fun [filter target filterer]
    [rec-list target [] (\ [e es] [if (filterer e) [cons e (es)] [(es)]])])

|
# Design Inspiration
## International Style a.k.a. Badmon Style



### Modernism & Grid formats


#### Neue Graphik Design

##### Fifth level, what if this was really long and we were able to cross over lines more than once. Lets try tha tby typig a lot here.
In a hole in the ground there lived a hobbit. Not a nasty, dirty, wet hole, filled with the ends of worms and an oozy smell, nor yet a dry, bare, sandy hole with nothing in it to sit down on or to eat: it was a hobbit-hole, and that means comfort.
###### Lowest Level





#### Notes


###### Colors

![munsell-color](/img/munsell-color.png)
We choose to stick to a plain black, white, and red color scheme to envoke the old school printer.<br><br> Colors **that** could be cool are red `#892B39` and linen `#F5F1E6`. International orange is another option: `#FF4F00`



- abc
- def

1. abc
1. def
"#;

    // let x = r#"|concat "abc"|"#;

    let md = bebop_lang::markdown::markdown_to_lisp(x).unwrap();
    println!("{}", md);
    let mut env = bebop_lang::lisp::env::init_env();

    let v = Lisp::from_source(&mut env, &md);
    println!("{:?}", v);
}
