	.globl main 
	.text 		
main:
	lw $t2, value3
	lw $t3, value1
	srl $t1, $t2, 3
test:
	jal test2
	addi $t2, $t1, 0
	lw $t4, value3
	lw $t5, 3(vec)
	sw $t5, 0x2(vec)
	bne $t1, $t3, test
test2:
	addi $t1, $t2, 0x1
	jr $ra

	.data
vec: .word 3, 5, 0, 1, 2
value0:	.word 0x55555555
value1:	.word 0x55557777
value2:	.word 0x1DA
value3:	.word 0x5B8
