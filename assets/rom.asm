#define .VRAM = 0x2000
#define .VRAMM = 0x2100
#section 0x4000
  LD   X,0x20
  LD   Y,X
.LOOP
  LD   .VRAM+X,Y
  INC  X
  LD   Y,X
  JMP NC,.LOOP
.END_L
  LD   Y,.END+X
  LD   .VRAMM+X,Y
  INC  X
  JMP NC,.END_L
  HLT
.END
  #byte 'END'
