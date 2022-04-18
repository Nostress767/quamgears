quamgears or a [QUArtus](https://en.wikipedia.org/wiki/Intel_Quartus_Prime) [MIPS](https://en.wikipedia.org/wiki/MIPS_architecture) Good Enough Assembler made in [RuSt](https://www.rust-lang.org/).
This was made to aid me in a university project, and is not in any way shape or form a complete product or assembler. In fact it only supports 15 instructions, that need to be written in a very specific way to be used in Quartus II 13 Web Edition:

(The commas(',') and spaces(' ') are necessary and strict in their placement)
These I-Format Instructions:
    SW, LW used like "instr reg, data_label"
    ADDI used like "addi reg1, reg2, immediate_value"
    BEQ, BNE used like "instr reg1, reg2, jump_label"
These R-Format Instructions:
    AND, OR, ADD, SUB, SLT used like "instr reg1, reg2, reg3"
    SRL, SLL used like "instr reg1, reg2, immediate_value"
    JR used like "jr reg"
These J-Format Instructions:
    J, JAL used like "instr jump_label"

Restriction:
The first 3 lines always have to be (or rather .global main always comes before .text that always comes before .data):
"""""""""""""""
	.globl main 
	.text 		
main:
"""""""""""""""

How to use:
--------------

    cargo run test_program.asm

To assemble the file, printing the result to stdout. The result includes both the program instructions and program data, so make sure to separate them yourself. Replace test_program.asm with any other file and/or redirect to a file (e.g.: cargo run test_program.asm > result.txt).

