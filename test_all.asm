	.globl main 
	.text 		
main:
	lw $t0, value1        # $t0 = value1 (0x12345678)
	srl $t1, $t0, 16      # $t1 = 0x00001234
	sll $t2, $t1, 8       # $t2 = 0x00123400
	sw $t2, value2        # value2 = $t2(0x00123400)
	or $t3, $t2, $t1      # $t3 = 0x00123634
	and $t2, $t3, $t1     # $t2 = 0x00001234
	add $t4, $t3, $t1     # $t4 = 0x00124868
	addi $t2, $t1, 1	  # $t2 = $t1 + 1
	slt $t2, $t1, $t2     # $t2 = 1 (t1 < t2)
	beq $t2, $at, skip
come_back:
	jal add_0x1234
skip:
	bne $t3, $t4, come_back
end:
	j end
add_0x1234:
	sub $t4, $t4, $t1     # $t4 = $t4 - $t1
	jr $ra

	.data
value1:	.word 0x12345678
value2:	.space
