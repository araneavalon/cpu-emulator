// // Hardware Constants
// #define .INTERRUPTP = 0x0000
// #define .BREAKP = 0x0002
// #define .BANK = 0x1000
// 
// // Kernel Constants (Generally in 0x0000-0x0BFF)
// #define .KEYBUFFERLEN = 0x0005 // Max 16
// #define .SCREENBUFFERP = 0x0006
// 
// #define .KEYBUFFER = 0x0BF0 // -0xBFF
// 
// // IO Constants
// #define .KEYBOARDCONTROL = 0x1FFF
// #define .KEYBOARDDATA = 0x1FFE
// 
// #define .SCREENCONTROL = 0x1FFD
// #define .SCREENCOMMAND = 0x1FFC
// #define .SCREENDATA = 0x1FFB
// 
// // Start of Bank 2 (Kernel Rom)
// #section 0x2000
// .KERNELINIT
//   LD   A,>.INTERRUPT
//   LD   >.INTERRUPTP,A
//   LD   A,<.INTERRUPT
//   LD   <.INTERRUPTP,A
//   // TODO SET BREAK HANDLER
//   CALL .SCREENINIT
//   SET  I,1 // Enable Interrupts
//   HLT
// 
// //
// // Screen Driver
// //
// .SCREENINIT
//   LD   X,0b00000001
//   LD   .SCREENCONTROL,X // Set font to 6x8
//   LD   X,0x00 // Address L
//   LD   Y,0x00 // Address H
//   LD   A,0x40 // Set Text Home
//   CALL .SCREENCALL2
//   LD   A,0x80 // Mode Set (CGROM, OR)
//   CALL .SCREENCALL0
//   LD   A,0x96 // Display Mode (Text Only, Blink Off)
//   CALL .SCREENCALL0
//   LD   A,0xA0 // 1-Line Cursor
//   CALL .SCREENCALL0
//   LD   X,0x01 // Enable
//   LD   Y,0x00 // Junk Data
//   LD   A,0x60 // Set Cursor Auto Move 
//   CALL .SCREENCALL0
//   RET
// //
// // Screen Call (0 params)
// // A,[B]
// //
// .SCREENCALL
// .SCREENCALL0
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENCOMMAND,A
// //
// // Screen Call (1 param)
// // A,X,[B]
// //
// .SCREENCALL1
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENDATA,X // Data 0
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENCOMMAND,A
// //
// // Screen Call (2 params)
// // A,X,Y,[B]
// //
// .SCREENCALL2
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENDATA,X // Data 0
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENDATA,Y // Data 1
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENCOMMAND,A
// //
// // Screen Auto Write
// // A,X,Y,[B] (Buffer Length, Screen Address L, Screen Address H)
// //
// .SCREENAUTOWRITE
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENDATA,X // Data 0
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENDATA,Y // Data 1
//   LD   X,A // Move Buffer Length to X
//   LD   A,0x24 // Set Address Pointer
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENCOMMAND,A
//   LD   A,0xB0 // Auto Write
//   LD   B,0x00000011
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENCOMMAND,A
// .SCREENAUTOWRITELOOP
//   DEC  X
//   LD   A,(.SCREENBUFFERP)+X
//   LD   B,.0x00001000
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENDATA,A
//   JMP NZ,.SCREENAUTOWRITELOOP
//   LD   A,0xB2 // Auto Reset
//   LD   B,.0x00001000
//   XOR  B,.SCREENCOMMAND
//   JMP  Z,*-8
//   LD   .SCREENCOMMAND,A
//   RET
// 
// 
// // Interrupt
// .INTERRUPT
//   PUSH A
//   LD   A,.KEYBOARDCONTROL
//   CMP  A,0b0000001
//   JMP  Z,.INTERRUPTEND
//   CALL .KEYBOARD
// .INTERRUPTEND
//   POP  A
//   RETI
// 
// //
// // Keyboard Driver
// // .KEYBOARDCONTROL (W) / Status (R)
// // .KEYBOARDDATA (RW)
// //
// .KEYBOARDINIT
//   // TODO KEYBOARDINIT
// .KEYBOARD
//   PUSH X
//   LD   X,.KEYBUFFERLEN
//   CMP  X,0x0F
//   JMP  C,.KEYBOARDLOAD
//   PUSH X
//   CALL .POPKEY
//   POP  X
// .KEYBOARDLOAD
//   LD   A,.KEYBOARDDATA
//   LD   (.KEYBUFFER)+X,A
//   INC  X
//   LD   .KEYBUFFERLEN,X
//   POP  X
//   RET
// 
// //
// // POPKEY
// // [B,X] -> A
// //
// .POPKEY
//   LD   A,(.KEYBUFFER)
//   LD   X,.KEYBUFFERLEN
//   DEC  X
//   LD   .KEYBUFFERLEN,X
// .POPKEYLOOP
//   LD   B,(.KEYBUFFER+1)+X
//   LD   (.KEYBUFFER)+X,B
//   DEC  X
//   JMP NC,.POPKEYLOOP
//   RET
// 
// 
// #section 0x4000  // Start of Bank 3 (Forth Rom)
// // Unfinished obviously
// // Not even started
// #word *+0x01
// #byte 0x01
// .hi #byte 'Hello!',0x01
// .end
// ld a,>.hi
// ld b,<.hi
// ld x,>.end
// ld y,<.end
// hlt
