target/inlining/speed/default-x64.o:	file format coff-x86-64

Disassembly of section .text:

0000000000000000 <neoclr_entry_v2>:
       0: 55                           	pushq	%rbp
       1: 48 89 e5                     	movq	%rsp, %rbp
       4: 48 83 ec 10                  	subq	$0x10, %rsp
       8: 45 33 c0                     	xorl	%r8d, %r8d
       b: 4d 89 c3                     	movq	%r8, %r11
       e: 4d 89 da                     	movq	%r11, %r10
      11: 41 81 c3 01 00 00 00         	addl	$0x1, %r11d
      18: 47 8d 04 d0                  	leal	(%r8,%r10,8), %r8d
      1c: 41 39 cb                     	cmpl	%ecx, %r11d
      1f: 0f 8c e9 ff ff ff            	jl	0xe <neoclr_entry_v2+0xe>
      25: 44 89 02                     	movl	%r8d, (%rdx)
      28: 33 c0                        	xorl	%eax, %eax
      2a: 48 83 c4 10                  	addq	$0x10, %rsp
      2e: 48 89 ec                     	movq	%rbp, %rsp
      31: 5d                           	popq	%rbp
      32: c3                           	retq

0000000000000033 <neoclr_scalar_1>:
      33: 55                           	pushq	%rbp
      34: 48 89 e5                     	movq	%rsp, %rbp
      37: 48 83 ec 10                  	subq	$0x10, %rsp
      3b: c1 e1 03                     	shll	$0x3, %ecx
      3e: 89 0a                        	movl	%ecx, (%rdx)
      40: 33 c0                        	xorl	%eax, %eax
      42: 48 83 c4 10                  	addq	$0x10, %rsp
      46: 48 89 ec                     	movq	%rbp, %rsp
      49: 5d                           	popq	%rbp
      4a: c3                           	retq
