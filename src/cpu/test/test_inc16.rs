use cpu::registers::Reg16;
use cpu::test::run_test;

fn test_inc16(opcode: u8, x: u16, reg: Reg16) -> bool {
  let cpu = run_test(
    &[opcode],
    |cpu| {
      cpu.regs.write16(reg, x);
    }
  );
  let expected = x.wrapping_add(1);
  cpu.clock_cycles() == 8 &&
    cpu.regs.read16(reg) == expected
}

#[quickcheck]
fn test_03(x: u16) -> bool {
  test_inc16(0x03, x, Reg16::BC)
}

#[test]
fn test_03_overflow() {
  assert!(test_inc16(0x03, 0xffff, Reg16::BC))
}

#[quickcheck]
fn test_13(x: u16) -> bool {
  test_inc16(0x13, x, Reg16::DE)
}

#[test]
fn test_13_overflow() {
  assert!(test_inc16(0x13, 0xffff, Reg16::DE))
}

#[quickcheck]
fn test_23(x: u16) -> bool {
  test_inc16(0x23, x, Reg16::HL)
}

#[test]
fn test_23_overflow() {
  assert!(test_inc16(0x23, 0xffff, Reg16::HL))
}

#[quickcheck]
fn test_33(x: u16) -> bool {
  test_inc16(0x33, x, Reg16::SP)
}

#[test]
fn test_33_overflow() {
  assert!(test_inc16(0x33, 0xffff, Reg16::SP))
}
