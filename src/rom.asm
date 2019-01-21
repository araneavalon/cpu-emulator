// Hardware Constants
#define .INTERRUPT_P = 0x0000
#define .BREAK_P = 0x0002
#define .BANK = 0x1000

// Kernel Constants (Generally in 0x0000-0x0BFF)
#define .KEY_BUFFER_LEN = 0x0005 // Max 16
#define .SCREEN_BUFFER_P = 0x0006

#define .KEY_BUFFER = 0x0BF0 // -0xBFF

// IO Constants
#define .KEYBOARD_CONTROL = 0x1FFF
#define .KEYBOARD_DATA = 0x1FFE

#define .SCREEN_CONTROL = 0x1FFD
#define .SCREEN_COMMAND = 0x1FFC
#define .SCREEN_DATA = 0x1FFB

// Start of Bank 2 (Kernel Rom)
#section 0x2000
.KERNEL_INIT
  LD   A,>.INTERRUPT
  LD   >.INTERRUPT_P,A
  LD   A,<.INTERRUPT
  LD   <.INTERRUPT_P,A
  // TODO SET BREAK HANDLER
  CALL .SCREEN_INIT
  SET  I,1 // Enable Interrupts
  HLT

//
// Screen Driver
//
.SCREEN_INIT
  LD   X,0b00000001
  LD   .SCREEN_CONTROL,X // Set font to 6x8
  LD   X,0x00 // Address L
  LD   Y,0x00 // Address H
  LD   A,0x40 // Set Text Home
  CALL .SCREEN_CALL_2
  LD   A,0x80 // Mode Set (CGROM, OR)
  CALL .SCREEN_CALL_0
  LD   A,0x96 // Display Mode (Text Only, Blink Off)
  CALL .SCREEN_CALL_0
  LD   A,0xA0 // 1-Line Cursor
  CALL .SCREEN_CALL_0
  LD   X,0x01 // Enable
  LD   Y,0x00 // Junk Data
  LD   A,0x60 // Set Cursor Auto Move 
  CALL .SCREEN_CALL_0
  RET
//
// Screen Call (0 params)
// A,[B]
//
.SCREEN_CALL
.SCREEN_CALL_0
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_COMMAND,A
//
// Screen Call (1 param)
// A,X,[B]
//
.SCREEN_CALL_1
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_DATA,X // Data 0
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_COMMAND,A
//
// Screen Call (2 params)
// A,X,Y,[B]
//
.SCREEN_CALL_2
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_DATA,X // Data 0
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_DATA,Y // Data 1
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_COMMAND,A
//
// Screen Auto Write
// A,X,Y,[B] (Buffer Length, Screen Address L, Screen Address H)
//
.SCREEN_AUTO_WRITE
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_DATA,X // Data 0
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_DATA,Y // Data 1
  LD   X,A // Move Buffer Length to X
  LD   A,0x24 // Set Address Pointer
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_COMMAND,A
  LD   A,0xB0 // Auto Write
  LD   B,0x00000011
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_COMMAND,A
.SCREEN_AUTO_WRITE_LOOP
  DEC  X
  LD   A,(.SCREEN_BUFFER_P)+X
  LD   B,.0x00001000
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_DATA,A
  JMP NZ,.SCREEN_AUTO_WRITE_LOOP
  LD   A,0xB2 // Auto Reset
  LD   B,.0x00001000
  XOR  B,.SCREEN_COMMAND
  JMP  Z,*-8
  LD   .SCREEN_COMMAND,A
  RET


// Interrupt
.INTERRUPT
  PUSH A
  LD   A,.KEYBOARD_CONTROL
  CMP  A,0b0000001
  JMP  Z,.INTERRUPT_END
  CALL .KEYBOARD
.INTERRUPT_END
  POP  A
  RETI

//
// Keyboard Driver
// .KEYBOARD_CONTROL (W) / Status (R)
// .KEYBOARD_DATA (RW)
//
.KEYBOARD_INIT
  // TODO KEYBOARD_INIT
.KEYBOARD
  PUSH X
  LD   X,.KEY_BUFFER_LEN
  CMP  X,0x0F
  JMP  C,.KEYBOARD_LOAD
  PUSH X
  CALL .POP_KEY
  POP  X
.KEYBOARD_LOAD
  LD   A,.KEYBOARD_DATA
  LD   (.KEY_BUFFER)+X,A
  INC  X
  LD   .KEY_BUFFER_LEN,X
  POP  X
  RET

//
// POP_KEY
// [B,X] -> A
//
.POP_KEY
  LD   A,(.KEY_BUFFER)
  LD   X,.KEY_BUFFER_LEN
  DEC  X
  LD   .KEY_BUFFER_LEN,X
.POP_KEY_LOOP
  LD   B,(.KEY_BUFFER+1)+X
  LD   (.KEY_BUFFER)+X,B
  DEC  X
  JMP NC,.POP_KEY_LOOP
  RET


#section 0x4000  // Start of Bank 3 (Forth Rom)
// Unfinished obviously
// Not even started
#word *+0x01
#byte 0x01
.hi #byte 'Hello!',0x01
.end
ld a,>.hi
ld b,<.hi
ld x,>.end
ld y,<.end
hlt
