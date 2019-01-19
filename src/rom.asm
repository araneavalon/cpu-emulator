#define .FF = 0xff
#section 0x2000  ; Start of Bank 2 (Kernel Rom)
JMP .end         ; Start of Forth ROM


#section 0x4000  ; Start of Bank 3 (Forth Rom)
; Unfinished obviously
; Not even started
#word *+0x01
#byte 0x01
.hi #byte 'Hello!',0x01
.end
ld a,>.hi
ld b,<.hi
ld x,>.end
ld y,<.end
hlt
