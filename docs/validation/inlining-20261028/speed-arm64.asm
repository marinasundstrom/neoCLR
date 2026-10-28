target/inlining/speed/default-arm64.o:	file format mach-o arm64

Disassembly of section __TEXT,__text:

0000000000000000 <_neoclr_entry_v2>:
       0: a9bf7bfd     	stp	x29, x30, [sp, #-0x10]!
       4: 910003fd     	mov	x29, sp
       8: d10043ff     	sub	sp, sp, #0x10
       c: 5280000c     	mov	w12, #0x0               ; =0
      10: aa0c03e2     	mov	x2, x12
      14: aa0203ef     	mov	x15, x2
      18: 11000442     	add	w2, w2, #0x1
      1c: 0b0f0d8c     	add	w12, w12, w15, lsl #3
      20: 6b00005f     	cmp	w2, w0
      24: 54ffff8b     	b.lt	0x14 <_neoclr_entry_v2+0x14>
      28: b900002c     	str	w12, [x1]
      2c: 52800000     	mov	w0, #0x0                ; =0
      30: 910043ff     	add	sp, sp, #0x10
      34: a8c17bfd     	ldp	x29, x30, [sp], #0x10
      38: d65f03c0     	ret

000000000000003c <_neoclr_scalar_1>:
      3c: a9bf7bfd     	stp	x29, x30, [sp, #-0x10]!
      40: 910003fd     	mov	x29, sp
      44: d10043ff     	sub	sp, sp, #0x10
      48: 531d7008     	lsl	w8, w0, #3
      4c: b9000028     	str	w8, [x1]
      50: 52800000     	mov	w0, #0x0                ; =0
      54: 910043ff     	add	sp, sp, #0x10
      58: a8c17bfd     	ldp	x29, x30, [sp], #0x10
      5c: d65f03c0     	ret
