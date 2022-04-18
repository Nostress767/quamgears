	.globl main 
	.text 		
main:
	lw $t2, value3
	lw $t3, value1
	srl $t1, $t2, 3
test:
	jal test2 # (absolute)
	addi $t2, $t1, 0
	lw $t4, value3
	bne $t1, $t3, test # (relative)
test2:
	addi $t1, $t2, 0x01000000 # 0x1 (big endian)
	jr $ra

	.data # data is litte endian, so bytes 0,1,2,3 (e.g.: .word 0x12345678 gets lw'd as 0x78563412)
value0:	.word 0x55555555
value1:	.word 0x77775555
value2:	.word 0xDA010000
value3:	.word 0xB8050000
#Z:	.word 0
