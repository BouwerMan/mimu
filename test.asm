li $t0, 10
li $t1, 20
li $t2, 40
loop:
add $t1, $t0, $t1
bne $t1, $t2, loop

# Exit syscall
addi $v0, $zero, 10
syscall
