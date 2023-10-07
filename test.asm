; count to 100 loop
:loop
move a f
move byte b 1
add
move f c
move a c
move byte b 100
less
jump loop true
