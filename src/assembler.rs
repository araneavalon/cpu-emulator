#[macro_use]
extern crate nom;

use nom::{
  digit,
  hex_digit,
};


named!(nop,  tag_no_case!("NOP" ));
named!(hlt,  tag_no_case!("HLT" ));
named!(brk,  tag_no_case!("BRK" ));
named!(int,  tag_no_case!("INT" ));

named!(set,  tag_no_case!("SET" ));

named!(call, tag_no_case!("CALL"));
named!(ret,  tag_no_case!("RET" ));
named!(reti, tag_no_case!("RETI"));
named!(jmp,  tag_no_case!("JMP" ));

named!(add,  tag_no_case!("ADD" ));
named!(addc, tag_no_case!("ADDC"));
named!(sub,  tag_no_case!("SUB" ));
named!(subc, tag_no_case!("SUBC"));
named!(and,  tag_no_case!("AND" ));
named!(or,   tag_no_case!("OR"  ));
named!(xor,  tag_no_case!("XOR" ));
named!(cmp,  tag_no_case!("CMP" ));

named!(inc,  tag_no_case!("INC" ));
named!(dec,  tag_no_case!("DEC" ));
named!(not,  tag_no_case!("NOT" ));
named!(neg,  tag_no_case!("NEG" ));
named!(rr,   tag_no_case!("RR"  ));
named!(rrc,  tag_no_case!("RRC" ));
named!(rl,   tag_no_case!("RL"  ));
named!(rlc,  tag_no_case!("RLC" ));

named!(push, tag_no_case!("PUSH"));
named!(pop,  tag_no_case!("POP" ));
named!(ld,   tag_no_case!("LD"  ));


fn byte_from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}
named!(byte<&str, Result<u8, std::num::ParseIntError>>,
  alt!(
    map!(recognize!(pair!(opt!(one_of!("+-")), digit)), std::string::FromStr::from_str) |
    map!(recognize!(preceded!(opt!(tag!("0x")), hex_digit)), byte_from_hex)
  )
);

fn address_from_hex(input: &str) -> Result<u16, std::num::ParseIntError> {
  u16::from_str_radix(input, 16)
}
named!(address<&str, Result<u16, std::num::ParseIntError>>,
  map!(recognize!(preceeded!(opt!(tag!("0x")), hex_digit)), address_from_hex)
);

named!(register,
	alt!(
		tag_no_case!("A") |
		tag_no_case!("B") |
		tag_no_case!("X") |
		tag_no_case!("Y")
	)
);
named!(accumulator,
	alt!(tag_no_case!("A") | tag_no_case!("B"))
);
named!(index,
	alt!(tag_no_case!("X") | tag_no_case!("Y"))
);
