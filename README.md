# c64 emulator and assembler
my own cpu architechture and assembler for it  

# c64
* uses a fetch and execute cycle
* 14 general 64-bit registers
* has a stack

# assembler
parses assembly code by doing 3 passes
## pass 1
remove comments
## pass 2
actually assembly and translate keywords to binary, but when it comes to labels, if a label is used but hasnt been declared yet, then it gets pushed to a `mentioned_labels` hashmap with a byte offset associated to it. at the same time all declared labels are pushed to a `found_labels`.
## pass 3
iterate over every mentioned label and overrite address

# how to run
assembler must be run with 2 args  
* asm filepath relative to executable
* filepath output relative to executable

example:  
`assembler.exe test.asm out.bin`  
`assembler.exe "../test.asm" "bin/out.bin"`

c64 must be run with 1 arg
* binary filepath(relative to executable) to run

example:  
`c64.exe out.bin`  
`c64.exe "../out.bin"`
