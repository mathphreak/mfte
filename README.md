# MFTE, the magic-free text editor
## With mediocre power comes miniscule responsibility

[![Travis](https://img.shields.io/travis/mathphreak/mfte.svg?style=flat-square&label=UNIX+builds)](https://travis-ci.org/mathphreak/mfte)
[![AppVeyor](https://img.shields.io/appveyor/ci/mathphreak/mfte.svg?style=flat-square&label=Windows+build)](https://ci.appveyor.com/project/mathphreak/mfte)

Because I am terrible and bad at everything, I'm writing a text editor.
Vim and Emacs are too complicated. Nano does almost everything I want. (I don't have an ideological grievance against Electron, but I think using it for a text editor is silly.)

Also works on Windows. Apparently nobody has written a cross-platform TUI library for Rust yet. I should probably get on that.

RULES:
- Don't do magic. Automatically indenting your entire file for you is really cool, but remembering how to do that takes up space in your brain that would be better spent on other things, like how to indent your code.
- Don't be modal. Typing text should always (within reason) actually insert text.
- Don't replace the arrow keys with other things. That's silly.
- Don't contain LISP. It's a text editor, stupid.
- Don't require four separate keystrokes just to save. \<Esc\>:w\<ret\> is absurd.
- Hell, don't make me go spelunking for my \<Esc\> key, like, ever.
- If it's even **possible** to write something like [pianobar.el](https://github.com/agrif/pianobar.el) for MFTE, I have done something terrible. Like, yes, it's cool that Emacs can do that, but couldn't you just run pianobar outside of Emacs? Why does that need to be built into Emacs? This goes for 80% of other silly Emacs tricks, too.
- Don't have default settings that don't make sense.
- Let me undo! I like Nano, but I do not like this about Nano.
- Don't reimplement copy and paste. My OS has that already.

Powerful text editors tend to overcomplicate indentation. Nano doesn't do anything particularly clever at all. The rule is simple: assume the indentation level hasn't changed from the previous line and make it easy to add or remove a level.
Decorated block comments, Markdown lists, and other line prefixes that aren't purely whitespace count too.
