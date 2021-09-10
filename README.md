Following along with [crafting interpreters](https://craftinginterpreters.com/), making a tree-walk interpreter for Lox in rust (instead of java).

It's fun, decided to skip a second pass for the environment resolving, and just keep the closures with the functions and pass through for evaluation.

Was a good learning journey, both the book and rust itself :)
Learning the Rc RefCell combination was very useful.

The interpreter itself looks to work ok.....

Very large recursive calls lead to segmentation fault tho.
Actual error messages are still a bit iffy.
In general very happy with the result, close enough for a bit of learning.

The book I can recommend very highly!