# MFTE, the magic-free text editor
## With mediocre power comes miniscule responsibility

[![Travis](https://img.shields.io/travis/mathphreak/mfte.svg?style=flat-square&label=UNIX+builds)](https://travis-ci.org/mathphreak/mfte)
[![AppVeyor](https://img.shields.io/appveyor/ci/mathphreak/mfte.svg?style=flat-square&label=Windows+build)](https://ci.appveyor.com/project/mathphreak/mfte)

Because I am terrible and bad at everything, I'm writing a text editor.
Vim and Emacs are too complicated. Nano does almost everything I want. (I don't have an ideological grievance against Electron, but I think using it for a text editor is silly.)

It's pronounced "mifty" (rhymes with "nifty").

Works best on Windows because that's what I use. Guaranteed to compile on Mac OS and Linux; full functionality might not happen.

## Configuration

In the interest of not reinventing the wheel, MFTE uses [EditorConfig](http://editorconfig.org/) to handle file-level settings like indent width and line endings.

In the interest of enabling my own laziness, not everything is implemented.
Indent size is just about the only thing that I care about different values of, and so I've tested 2 and 4 and those work fine. Other numerical values should work too.

The simpler things, like EOF behavior and the two boolean options, should work however they're set up, but I haven't exhaustively tested them.

Setting your indentation to tabs will break everything. (Loading files with indentation set to tabs will also break everything, come to think of it. This might not be a good thing.)

You can't change character sets. It's UTF-8. This is a feature.

## Guiding Development Principles

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

## OS compatibility

Windows has actual APIs for getting one keypress at a time and getting all modifier keys, so MFTE can receive (say) Ctrl+Tab or Shift+Down.
If I wanted Ctrl+Shift+S to be different from Ctrl+S, that would work too.

On UNIX, meanwhile, everything is garbage.
Tab is just Ctrl+I, so Ctrl+Tab is not even possible.
Down is Esc [ B, so Shift+Down also makes no sense.
I have no idea how to resolve this without writing my own terminal emulator just for MFTE.
Apparently you can shenanigans xterm into reinterpreting things as other things, so that might work.
