# Chris's Compiled Stack-Based Language

This is a toy stack based language, roughly following Forth's syntax.

It's part written, with a preliminary scanner in place and the assembler on its
way. There's nothing special or clever about it, it's just a learning exercise.

## Example

This example uses supported operations. Stack comments show the effect of each
line, with the top of the stack on the right; deeper values are unchanged.
Division truncates towards zero, and a non-zero remainder has the dividend's sign.

```ccl
1 2 add       # ( -- 3 )
print         # ( 3 -- )        displays 3
10 4 sub      # ( -- 6 )
pop           # ( 6 -- )        discards 6, nothing displayed

# Verify a = (a / b) * b + (a % b), using a = -7 and b = 3
-7 3 /        # ( -- -2 )       quotient truncated towards zero
3 *           # ( -2 -- -6 )
-7 3 %        # ( -- -1 )       leaves the earlier -6 underneath
+             # ( -6 -1 -- -7 )
print         # ( -7 -- )       displays -7
halt          # ( -- )          ends execution
```
