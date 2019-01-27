#define .VRAM = 0x2000

#define .BANK = 0x2F00
#define .KEY_STATUS = 0x2F01
#define .KEY_DATA = 0x2F02

#define .KEY_B_SP = 0x0FEF
// #define .KEY_B_H = 0x0FFF
#define .KEY_B_H = 0x200F
// #define .KEY_B = 0x0FF0
#define .KEY_B = 0x2000

#section 0x4000
// INIT
  LD A,>.INTERRUPT
  LD @0x0000,A
  LD A,<.INTERRUPT
  LD @0x0001,A
  LD A,0x0F
  LD @.KEY_B_SP,A

// Interrupt Handler
.INTERRUPT
  PUSH A
  LD   A,@.KEY_STATUS
  CMP  A,0
  JMP NZ,@.KEYBOARD
  POP  A
  RETI

// Keyboard Interrupt Handler
// (A) -> 
.KEYBOARD
  LD   A,@.KEY_DATA+1
  LD   (@.KEY_B_SP),A
  LD   A,@.KEY_B_SP
  DEC  A
  JMP  C,@.KEYBOARD_E
  LD   @.KEY_B_SP,A
.KEYBOARD_E
  RET

// Pop Char from Key Buffer
// () X,Y -> A
.KEY_POP
  LD   X,0x0E
  CMP  X,@.KEY_B_SP
  JMP  C,@.KEY_POP_F
  LD   A,@.KEY_B_H
.KEY_POP_L
  LD   Y,@.KEY_B+X
  LD   @.KEY_B+1+X,Y
  DEC  X
  JMP NZ,@.KEY_POP_L
  LD   X,@.KEY_B_SP
  INC  X
  LD   @.KEY_B_SP,X
  RET
.KEY_POP_F
  LD   A,0x00
  RET

