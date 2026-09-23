# Chris's Compiled Stack-Based Language

This is a toy stack based language, roughly following Forth's syntax.

It's part written, with a preliminary scanner in place and the assembler on its
way. There's nothing special or clever about it, it's just a learning exercise.

## Example

ccsbl doesn't support all of these operations yet but it will eventually.

```ccl
1 2 add       # ( -- 3 )
print         # ( 3 -- )        displays 3
10 4 sub      # ( -- 6 )
pop           # ( 6 -- )        discards 6, nothing displayed
1             # ( -- 1 )        initial counter
start:
dup add       # ( n -- 2n )     double the counter
dup 9 gt      # ( n -- n bool ) test n > 9
jt end
jmp start
end:
print         # ( n -- )        displays the final value (16)
halt
```
