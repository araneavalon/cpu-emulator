
use crate::control::microcode::{self, Microcode};
use crate::control::control::Control;


pub struct Instructions {
  fetch: Control,
  instructions: [Vec<Microcode>; 32],
  decode: [usize; 1024],
}

impl Instructions {
  pub fn new() -> Instructions {
    let microcode = microcode::array();
    Instructions {
      fetch: microcode[0].decode(0x0000), // Opcode doesn't matter for fetch.
      instructions: Instructions::array(microcode),
      decode: Instructions::decode_table(),
    }
  }

  fn array(microcode: [Microcode; 46]) -> [Vec<Microcode>; 32] {
    [
      vec![ // 0    OP r,be
        microcode[1],
        microcode[2],
        microcode[3],
      ],
      vec![ // 1   OPw r,(r+r)
        microcode[4],
        microcode[5],
        microcode[6],
        microcode[7],
        microcode[8],
        microcode[9],
      ],
      vec![ // 2   OPw r,(s+b)
        microcode[10],
        microcode[11],
        microcode[6],
        microcode[7],
        microcode[8],
        microcode[9],
      ],
      vec![ // 3   OP r,r
        microcode[7],
        microcode[12],
        microcode[9],
      ],
      vec![ // 4   OPw r,(r)
        microcode[7],
        microcode[13],
        microcode[8],
        microcode[9],
      ],
      vec![ // 5   OPw r,word
        microcode[7],
        microcode[14],
        microcode[9],
      ],
      vec![ // 6   OPw r,(word)
        microcode[7],
        microcode[15],
        microcode[8],
        microcode[9],
      ],

      vec![ // 7   UOP r
        microcode[16],
        microcode[17],
      ],

      vec![ // 8   LD r,be
        microcode[34],
      ],
      vec![ // 9   LDw r,(r+r) &r
        microcode[4],
        microcode[5],
        microcode[6],
        microcode[35],
      ],
      vec![ // 10  LD r,(s+b) &r
        microcode[10],
        microcode[11],
        microcode[6],
        microcode[35],
      ],
      vec![ // 11  LD r,r
        microcode[36],
      ],
      vec![ // 12  LD r,(r) &r
        microcode[37],
        microcode[35],
      ],
      vec![ // 13  LD r,word
        microcode[38],
      ],
      vec![ // 14  LD r,(word) &r
        microcode[39],
        microcode[35],
      ],

      vec![ // 15  LD x,r &r
        microcode[40],
      ],
      vec![ // 16  LD x,(r) &r
        microcode[37],
        microcode[41],
      ],
      vec![ // 17  LD x,word
        microcode[42],
      ],
      vec![ // 18  LD x,(word) &r
        microcode[37],
        microcode[41],
      ],

      vec![ // 19  JMP(L) b
        microcode[18],
        microcode[19],
        microcode[20],
      ],
      vec![ // 20  JMP LR
        microcode[26],
      ],
      vec![ // 21  POP(L)s PC
        microcode[27],
      ],
      vec![ // 22  JMP(L) r
        microcode[21],
      ],
      vec![ // 23  JMP(L) (r)
        microcode[22],
        microcode[23],
      ],
      vec![ // 24  JMP(L) word
        microcode[24],
      ],
      vec![ // 25  JMP(L) (word)
        microcode[25],
        microcode[23],
      ],

      vec![ // 26  SET r,b,v
        microcode[28],
        microcode[29],
        microcode[30],
      ],
      vec![ // 27  SET F,b,v
        microcode[31],
        microcode[29],
        microcode[32],
      ],
      vec![ // 28  TEST r,b
        microcode[28],
        microcode[29],
        microcode[33],
      ],

      vec![ // 29 PUTs,POPs
        microcode[43],
      ],

      vec![ // 30  INT b / BRK b
        microcode[44],
      ],
      vec![ // 31  NOP / HLT
        microcode[45],
      ],
    ]
  }

  fn decode_table() -> [usize; 1024] {
    [
      31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
      30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
      31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
      31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
      29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29,
      29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29,
      29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29,
      29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29,
      14, 13, 12, 11, 18, 17, 10, 10, 14, 13, 12, 11, 16, 15, 10, 10,
      14, 13, 12, 11, 18, 17, 10, 10, 14, 13, 12, 11, 16, 15, 10, 10,
       9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,
       9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,  9,
       8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,
       8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,
       8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,
       8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,
      28, 28, 28, 28, 28, 28, 28, 28, 20, 20, 24, 25, 22, 23, 21, 21,
      28, 28, 28, 28, 28, 28, 28, 28, 20, 20, 24, 25, 22, 23, 21, 21,
      28, 28, 28, 28, 28, 28, 28, 28, 20, 20, 24, 25, 22, 23, 21, 21,
      28, 28, 28, 28, 28, 28, 28, 28, 20, 20, 24, 25, 22, 23, 21, 21,
      26, 26, 26, 26, 27, 27, 27, 27, 20, 20, 24, 25, 22, 23, 21, 21,
      26, 26, 26, 26, 27, 27, 27, 27, 20, 20, 24, 25, 22, 23, 21, 21,
      26, 26, 26, 26, 27, 27, 27, 27, 20, 20, 24, 25, 22, 23, 21, 21,
      26, 26, 26, 26, 27, 27, 27, 27, 20, 20, 24, 25, 22, 23, 21, 21,
      19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
      19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
      19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
      19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
      19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
      19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
      19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
      19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
       5,  6,  3,  4,  7,  7,  2,  2,  5,  6,  3,  4,  7,  7,  2,  2,
       1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
       5,  6,  3,  4,  7,  7,  2,  2,  5,  6,  3,  4,  7,  7,  2,  2,
       1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
       5,  6,  3,  4,  7,  7,  2,  2,  5,  6,  3,  4,  7,  7,  2,  2,
       1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
       5,  6,  3,  4,  7,  7,  2,  2,  5,  6,  3,  4,  7,  7,  2,  2,
       1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
       5,  6,  3,  4,  7,  7,  2,  2,  5,  6,  3,  4,  7,  7,  2,  2,
       1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
       5,  6,  3,  4,  7,  7,  2,  2,  5,  6,  3,  4,  7,  7,  2,  2,
       1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
       5,  6,  3,  4,  7,  7,  2,  2,  5,  6,  3,  4,  7,  7,  2,  2,
       1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
       5,  6,  3,  4,  7,  7,  2,  2,  5,  6,  3,  4,  7,  7,  2,  2,
       1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
       0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    ]
  }

  pub fn fetch(&self) -> Control {
    self.fetch
  }

  pub fn get(&self, op: u16) -> &Vec<Microcode> {
    &self.instructions[self.decode[(op >> 6) as usize]]
  }
}
