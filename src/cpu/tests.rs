use super::*;
use AddressingMode::*;
use Mnemonic::*;

/* Helpers */

/// Look up an opcode byte from OPCODES table, avoids hardcoding const hexes
fn op(mnemonic: Mnemonic, mode: AddressingMode) -> u8 {
    OPCODES
        .iter()
        .position(|entry| matches!(entry, Some(i) if i.mnemonic == mnemonic && i.mode == mode))
        .expect("unknown opcode") as u8
}

fn run(program: Vec<u8>) -> Cpu {
    run_seeded(program, &[])
}

fn run_seeded(program: Vec<u8>, seed: &[(u16, u8)]) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.load(program);
    cpu.reset(); // sets PC from the reset vector that load() wrote
    for &(addr, value) in seed {
        cpu.mem_write(addr, value);
    }
    cpu.run();
    cpu
}

/// Runs `LDX #$42; <setup>; <branch> +2; LDX #$FF; BRK`.
///
/// The branch's target is the final BRK, so a taken branch skips the
/// `LDX #$FF` and leaves X = 0x42; a non-taken branch falls through to it and
/// leaves X = 0xFF. X is the sentinel because the flag `setup` is free to
/// clobber A (the overflow setup pushes/pulls through it).
fn run_branch(setup: &[u8], mnemonic: Mnemonic) -> Cpu {
    let mut program = vec![op(LDX, Immediate), 0x42];
    program.extend_from_slice(setup);
    program.extend_from_slice(&[
        op(mnemonic, Relative),
        0x02, // skip the 2-byte LDX #$FF that follows
        op(LDX, Immediate),
        0xFF,
        op(BRK, Implied),
    ]);
    run(program)
}

mod lda {
    use super::*;

    #[test]
    fn immediate() {
        let cpu = run(vec![op(LDA, Immediate), 0x05, op(BRK, Implied)]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn immediate_zero_flag() {
        let cpu = run(vec![op(LDA, Immediate), 0x00, op(BRK, Implied)]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn immediate_negative_flag() {
        let cpu = run(vec![op(LDA, Immediate), 0x80, op(BRK, Implied)]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(!cpu.status.zero);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![op(LDA, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn zero_page_x() {
        // X = 1, base = 0x10 -> reads 0x11
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x01,
                op(LDA, ZeroPageX),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x11, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn zero_page_x_wraps_in_zero_page() {
        // base = 0xFF, X = 0x12 -> wraps to 0x11 (not 0x111). The wrap target is
        // kept above the program (loaded at 0x0000) so it doesn't alias the code.
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x12,
                op(LDA, ZeroPageX),
                0xFF,
                op(BRK, Implied),
            ],
            &[(0x11, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn absolute() {
        let cpu = run_seeded(
            vec![op(LDA, Absolute), 0x34, 0x12, op(BRK, Implied)],
            &[(0x1234, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn absolute_x() {
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x01,
                op(LDA, AbsoluteX),
                0x34,
                0x12,
                op(BRK, Implied),
            ],
            &[(0x1235, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn absolute_y() {
        let cpu = run_seeded(
            vec![
                op(LDY, Immediate),
                0x01,
                op(LDA, AbsoluteY),
                0x34,
                0x12,
                op(BRK, Implied),
            ],
            &[(0x1235, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn indirect_x() {
        // X = 1, zp operand 0x10 -> pointer read from 0x11/0x12 = 0x0700
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x01,
                op(LDA, IndirectX),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x11, 0x00), (0x12, 0x07), (0x0700, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn indirect_y() {
        // pointer at 0x10/0x11 = 0x0700, then + Y(4) = 0x0704
        let cpu = run_seeded(
            vec![
                op(LDY, Immediate),
                0x04,
                op(LDA, IndirectY),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x00), (0x11, 0x07), (0x0704, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }
}

mod ldx {
    use super::*;

    #[test]
    fn immediate() {
        let cpu = run(vec![op(LDX, Immediate), 0x05, op(BRK, Implied)]);
        assert_eq!(cpu.register_x, 0x05);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn immediate_zero_flag() {
        let cpu = run(vec![op(LDX, Immediate), 0x00, op(BRK, Implied)]);
        assert!(cpu.status.zero);
    }

    #[test]
    fn immediate_negative_flag() {
        let cpu = run(vec![op(LDX, Immediate), 0x80, op(BRK, Implied)]);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![op(LDX, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x42)],
        );
        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn zero_page_y() {
        let cpu = run_seeded(
            vec![
                op(LDY, Immediate),
                0x01,
                op(LDX, ZeroPageY),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x11, 0x42)],
        );
        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn absolute() {
        let cpu = run_seeded(
            vec![op(LDX, Absolute), 0x34, 0x12, op(BRK, Implied)],
            &[(0x1234, 0x42)],
        );
        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn absolute_y() {
        let cpu = run_seeded(
            vec![
                op(LDY, Immediate),
                0x01,
                op(LDX, AbsoluteY),
                0x34,
                0x12,
                op(BRK, Implied),
            ],
            &[(0x1235, 0x42)],
        );
        assert_eq!(cpu.register_x, 0x42);
    }
}

mod ldy {
    use super::*;

    #[test]
    fn immediate() {
        let cpu = run(vec![op(LDY, Immediate), 0x05, op(BRK, Implied)]);
        assert_eq!(cpu.register_y, 0x05);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn immediate_zero_flag() {
        let cpu = run(vec![op(LDY, Immediate), 0x00, op(BRK, Implied)]);
        assert!(cpu.status.zero);
    }

    #[test]
    fn immediate_negative_flag() {
        let cpu = run(vec![op(LDY, Immediate), 0x80, op(BRK, Implied)]);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![op(LDY, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x42)],
        );
        assert_eq!(cpu.register_y, 0x42);
    }

    #[test]
    fn zero_page_x() {
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x01,
                op(LDY, ZeroPageX),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x11, 0x42)],
        );
        assert_eq!(cpu.register_y, 0x42);
    }

    #[test]
    fn absolute() {
        let cpu = run_seeded(
            vec![op(LDY, Absolute), 0x34, 0x12, op(BRK, Implied)],
            &[(0x1234, 0x42)],
        );
        assert_eq!(cpu.register_y, 0x42);
    }

    #[test]
    fn absolute_x() {
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x01,
                op(LDY, AbsoluteX),
                0x34,
                0x12,
                op(BRK, Implied),
            ],
            &[(0x1235, 0x42)],
        );
        assert_eq!(cpu.register_y, 0x42);
    }
}

mod sta {
    use super::*;

    #[test]
    fn zero_page() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(STA, ZeroPage),
            0x10,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x10), 0x42);
    }

    #[test]
    fn zero_page_x() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(LDX, Immediate),
            0x01,
            op(STA, ZeroPageX),
            0x10,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x11), 0x42);
    }

    #[test]
    fn absolute() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(STA, Absolute),
            0x34,
            0x12,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x1234), 0x42);
    }

    #[test]
    fn absolute_x() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(LDX, Immediate),
            0x01,
            op(STA, AbsoluteX),
            0x34,
            0x12,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x1235), 0x42);
    }

    #[test]
    fn absolute_y() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(LDY, Immediate),
            0x01,
            op(STA, AbsoluteY),
            0x34,
            0x12,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x1235), 0x42);
    }

    #[test]
    fn indirect_x() {
        // pointer at 0x11/0x12 = 0x0700
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x42,
                op(LDX, Immediate),
                0x01,
                op(STA, IndirectX),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x11, 0x00), (0x12, 0x07)],
        );
        assert_eq!(cpu.mem_read(0x0700), 0x42);
    }

    #[test]
    fn indirect_y() {
        // pointer at 0x10/0x11 = 0x0700, + Y(4) = 0x0704
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x42,
                op(LDY, Immediate),
                0x04,
                op(STA, IndirectY),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x00), (0x11, 0x07)],
        );
        assert_eq!(cpu.mem_read(0x0704), 0x42);
    }
}

mod stx {
    use super::*;

    #[test]
    fn zero_page() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x42,
            op(STX, ZeroPage),
            0x10,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x10), 0x42);
    }

    #[test]
    fn zero_page_y() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x42,
            op(LDY, Immediate),
            0x01,
            op(STX, ZeroPageY),
            0x10,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x11), 0x42);
    }

    #[test]
    fn absolute() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x42,
            op(STX, Absolute),
            0x34,
            0x12,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x1234), 0x42);
    }
}

mod sty {
    use super::*;

    #[test]
    fn zero_page() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x42,
            op(STY, ZeroPage),
            0x10,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x10), 0x42);
    }

    #[test]
    fn zero_page_x() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x42,
            op(LDX, Immediate),
            0x01,
            op(STY, ZeroPageX),
            0x10,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x11), 0x42);
    }

    #[test]
    fn absolute() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x42,
            op(STY, Absolute),
            0x34,
            0x12,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x1234), 0x42);
    }
}

mod tax {
    use super::*;

    #[test]
    fn transfers_a_to_x() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(TAX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn sets_zero_flag() {
        // A = 0; clobber X (and clear the zero flag) before TAX, so the test
        // proves TAX itself sets zero, not the preceding LDA.
        let cpu = run(vec![
            op(LDA, Immediate),
            0x00,
            op(LDX, Immediate),
            0x01,
            op(TAX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x80,
            op(LDX, Immediate),
            0x01,
            op(TAX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x80);
        assert!(cpu.status.negative);
    }
}

mod tay {
    use super::*;

    #[test]
    fn transfers_a_to_y() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(TAY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0x42);
    }

    #[test]
    fn sets_zero_flag() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x00,
            op(LDY, Immediate),
            0x01,
            op(TAY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x80,
            op(LDY, Immediate),
            0x01,
            op(TAY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0x80);
        assert!(cpu.status.negative);
    }
}

mod txa {
    use super::*;

    #[test]
    fn transfers_x_to_a() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x42,
            op(TXA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn sets_zero_flag() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x00,
            op(LDA, Immediate),
            0x01,
            op(TXA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x80,
            op(LDA, Immediate),
            0x01,
            op(TXA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(cpu.status.negative);
    }
}

mod tya {
    use super::*;

    #[test]
    fn transfers_y_to_a() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x42,
            op(TYA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn sets_zero_flag() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x00,
            op(LDA, Immediate),
            0x01,
            op(TYA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x80,
            op(LDA, Immediate),
            0x01,
            op(TYA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(cpu.status.negative);
    }
}

mod tsx {
    use super::*;

    // TSX reads the stack pointer, which can only be set via TXS, so each test
    // seeds S with `LDX; TXS` first. This keeps them independent of whatever
    // the power-on stack pointer value is.

    #[test]
    fn transfers_stack_pointer_to_x() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x42,
            op(TXS, Implied), // S = 0x42
            op(LDX, Immediate),
            0x00,             // clobber X
            op(TSX, Implied), // X = S = 0x42
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn sets_zero_flag() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x00,
            op(TXS, Implied), // S = 0
            op(LDX, Immediate),
            0x01,             // clear zero flag
            op(TSX, Implied), // X = 0
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x80,
            op(TXS, Implied), // S = 0x80
            op(LDX, Immediate),
            0x01,             // clear negative flag
            op(TSX, Implied), // X = 0x80
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x80);
        assert!(cpu.status.negative);
    }
}

mod txs {
    use super::*;

    #[test]
    fn transfers_x_to_stack_pointer() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x42,
            op(TXS, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.stack_pointer, 0x42);
    }

    #[test]
    fn does_not_affect_flags() {
        // X = 0x80 (which would set negative if TXS wrongly touched flags), then
        // clear the flags with LDA #$01 before TXS. Flags must reflect the LDA,
        // not the transfer.
        let cpu = run(vec![
            op(LDX, Immediate),
            0x80,
            op(LDA, Immediate),
            0x01,
            op(TXS, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.stack_pointer, 0x80);
        assert!(!cpu.status.negative);
        assert!(!cpu.status.zero);
    }
}

mod pha {
    use super::*;

    #[test]
    fn pushes_accumulator_to_stack() {
        // S starts at 0xFD after reset, so the push lands at 0x01FD.
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(PHA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.mem_read(0x01FD), 0x42);
    }

    #[test]
    fn decrements_stack_pointer() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(PHA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.stack_pointer, 0xFC);
    }
}

mod pla {
    use super::*;

    #[test]
    fn pulls_into_accumulator() {
        // Push 0x42, clobber A, then pull it back.
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(PHA, Implied),
            op(LDA, Immediate),
            0x00,
            op(PLA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn increments_stack_pointer() {
        // push then pull returns S to its starting value.
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(PHA, Implied),
            op(PLA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.stack_pointer, 0xFD);
    }

    #[test]
    fn sets_zero_flag() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x00,
            op(PHA, Implied),
            op(LDA, Immediate),
            0x01, // clear zero flag before the pull
            op(PLA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x80,
            op(PHA, Implied),
            op(LDA, Immediate),
            0x01, // clear negative flag before the pull
            op(PLA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(cpu.status.negative);
    }
}

mod php {
    use super::*;

    #[test]
    fn pushes_status_with_break_and_bit5_set() {
        // Fresh status is all-clear; PHP must still push bit 4 (B) and bit 5 set,
        // i.e. 0x30. Read the pushed byte back with PLA.
        let cpu = run(vec![op(PHP, Implied), op(PLA, Implied), op(BRK, Implied)]);
        assert_eq!(cpu.register_a, 0x30);
    }

    #[test]
    fn reflects_current_flags() {
        // LDA #$80 sets the negative flag (bit 7). Pushed byte = 0x80|0x30 = 0xB0.
        let cpu = run(vec![
            op(LDA, Immediate),
            0x80,
            op(PHP, Implied),
            op(PLA, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0xB0);
    }
}

mod plp {
    use super::*;

    #[test]
    fn restores_all_flags() {
        // Push 0xFF, then PLP should set every real flag.
        let cpu = run(vec![
            op(LDA, Immediate),
            0xFF,
            op(PHA, Implied),
            op(PLP, Implied),
            op(BRK, Implied),
        ]);
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
        assert!(cpu.status.interrupt);
        assert!(cpu.status.decimal);
        assert!(cpu.status.overflow);
        assert!(cpu.status.negative);
    }

    #[test]
    fn clears_all_flags() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x00,
            op(PHA, Implied),
            op(PLP, Implied),
            op(BRK, Implied),
        ]);
        assert!(!cpu.status.carry);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.interrupt);
        assert!(!cpu.status.decimal);
        assert!(!cpu.status.overflow);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn ignores_break_and_bit5() {
        // 0x30 has only bits 4 and 5 set, which are not real flags. PLP must
        // discard them, leaving every flag clear.
        let cpu = run(vec![
            op(LDA, Immediate),
            0x30,
            op(PHA, Implied),
            op(PLP, Implied),
            op(BRK, Implied),
        ]);
        assert!(!cpu.status.carry);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.interrupt);
        assert!(!cpu.status.decimal);
        assert!(!cpu.status.overflow);
        assert!(!cpu.status.negative);
    }
}

mod and {
    use super::*;

    #[test]
    fn immediate() {
        // 0b1100 & 0b1010 = 0b1000
        let cpu = run(vec![
            op(LDA, Immediate),
            0b1100,
            op(AND, Immediate),
            0b1010,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0b1000);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_zero_flag() {
        // disjoint bits AND to zero
        let cpu = run(vec![
            op(LDA, Immediate),
            0x0F,
            op(AND, Immediate),
            0xF0,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x80,
            op(AND, Immediate),
            0x80,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0xFF,
                op(AND, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x0F)],
        );
        assert_eq!(cpu.register_a, 0x0F);
    }
}

mod eor {
    use super::*;

    #[test]
    fn immediate() {
        // 0b1100 ^ 0b1010 = 0b0110
        let cpu = run(vec![
            op(LDA, Immediate),
            0b1100,
            op(EOR, Immediate),
            0b1010,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0b0110);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_zero_flag() {
        // a value XORed with itself is zero
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(EOR, Immediate),
            0x42,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        // 0x00 ^ 0x80 = 0x80
        let cpu = run(vec![
            op(LDA, Immediate),
            0x00,
            op(EOR, Immediate),
            0x80,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0xFF,
                op(EOR, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x0F)],
        );
        assert_eq!(cpu.register_a, 0xF0);
    }
}

mod ora {
    use super::*;

    #[test]
    fn immediate() {
        // 0b1100 | 0b1010 = 0b1110
        let cpu = run(vec![
            op(LDA, Immediate),
            0b1100,
            op(ORA, Immediate),
            0b1010,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0b1110);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_zero_flag() {
        // only 0 | 0 is zero
        let cpu = run(vec![
            op(LDA, Immediate),
            0x00,
            op(ORA, Immediate),
            0x00,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x01,
            op(ORA, Immediate),
            0x80,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x81);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x0F,
                op(ORA, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0xF0)],
        );
        assert_eq!(cpu.register_a, 0xFF);
    }
}

mod bit {
    use super::*;

    #[test]
    fn sets_zero_when_no_common_bits() {
        // A & value == 0 -> zero set, but A is unchanged.
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x0F,
                op(BIT, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0xF0)],
        );
        assert_eq!(cpu.register_a, 0x0F); // A is read-only
        assert!(cpu.status.zero);
    }

    #[test]
    fn clears_zero_when_common_bits() {
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0xFF,
                op(BIT, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x01)],
        );
        assert!(!cpu.status.zero);
    }

    #[test]
    fn negative_comes_from_operand_not_and() {
        // A = 0, so A & value == 0, yet bit 7 of the value still sets negative.
        // This proves N reflects the operand, not the masked result.
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x00,
                op(BIT, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x80)],
        );
        assert!(cpu.status.zero); // 0 & 0x80 == 0
        assert!(cpu.status.negative); // bit 7 of the operand
    }

    #[test]
    fn overflow_comes_from_operand_bit6() {
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x00,
                op(BIT, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x40)],
        );
        assert!(cpu.status.overflow);
    }

    #[test]
    fn clears_negative_and_overflow() {
        // operand with bits 6 and 7 clear leaves both flags clear
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0xFF,
                op(BIT, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x3F)],
        );
        assert!(!cpu.status.negative);
        assert!(!cpu.status.overflow);
    }

    #[test]
    fn absolute() {
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0xC0,
                op(BIT, Absolute),
                0x34,
                0x12,
                op(BRK, Implied),
            ],
            &[(0x1234, 0xC0)],
        );
        assert!(!cpu.status.zero); // 0xC0 & 0xC0 != 0
        assert!(cpu.status.negative); // bit 7
        assert!(cpu.status.overflow); // bit 6
    }
}

mod adc {
    use super::*;

    #[test]
    fn adds_with_carry_clear() {
        // CLC so no carry-in: 0x10 + 0x20 = 0x30
        let cpu = run(vec![
            op(CLC, Implied),
            op(LDA, Immediate),
            0x10,
            op(ADC, Immediate),
            0x20,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x30);
        assert!(!cpu.status.carry);
        assert!(!cpu.status.overflow);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn adds_carry_in() {
        // SEC adds an extra 1: 0x10 + 0x20 + 1 = 0x31
        let cpu = run(vec![
            op(SEC, Implied),
            op(LDA, Immediate),
            0x10,
            op(ADC, Immediate),
            0x20,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x31);
    }

    #[test]
    fn sets_carry_and_zero_on_wrap() {
        // 0xFF + 0x01 = 0x100 -> result 0x00, carry out
        let cpu = run(vec![
            op(CLC, Implied),
            op(LDA, Immediate),
            0xFF,
            op(ADC, Immediate),
            0x01,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_overflow_on_signed_wrap() {
        // 0x50 + 0x50 = 0xA0: two positives sum to a negative, so V is set.
        let cpu = run(vec![
            op(CLC, Implied),
            op(LDA, Immediate),
            0x50,
            op(ADC, Immediate),
            0x50,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0xA0);
        assert!(cpu.status.overflow);
        assert!(cpu.status.negative);
        assert!(!cpu.status.carry);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![
                op(CLC, Implied),
                op(LDA, Immediate),
                0x10,
                op(ADC, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x20)],
        );
        assert_eq!(cpu.register_a, 0x30);
    }
}

mod sbc {
    use super::*;

    // SBC computes A - M - (1 - C), so a borrowless subtraction needs SEC first.

    #[test]
    fn subtracts_with_carry_set() {
        // SEC clears the borrow: 0x50 - 0x10 = 0x40, carry stays set (no borrow)
        let cpu = run(vec![
            op(SEC, Implied),
            op(LDA, Immediate),
            0x50,
            op(SBC, Immediate),
            0x10,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x40);
        assert!(cpu.status.carry);
        assert!(!cpu.status.overflow);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn borrows_when_carry_clear() {
        // CLC injects a borrow: 0x50 - 0x10 - 1 = 0x3F
        let cpu = run(vec![
            op(CLC, Implied),
            op(LDA, Immediate),
            0x50,
            op(SBC, Immediate),
            0x10,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x3F);
        assert!(cpu.status.carry); // no borrow out of this subtraction
    }

    #[test]
    fn clears_carry_on_borrow() {
        // 0x50 - 0x60 underflows to 0xF0; carry clears to signal the borrow.
        let cpu = run(vec![
            op(SEC, Implied),
            op(LDA, Immediate),
            0x50,
            op(SBC, Immediate),
            0x60,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0xF0);
        assert!(!cpu.status.carry);
        assert!(cpu.status.negative);
    }

    #[test]
    fn sets_zero_when_equal() {
        let cpu = run(vec![
            op(SEC, Implied),
            op(LDA, Immediate),
            0x50,
            op(SBC, Immediate),
            0x50,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.zero);
        assert!(cpu.status.carry);
    }

    #[test]
    fn sets_overflow_on_signed_wrap() {
        // 0x50 - 0xB0: positive minus negative yielding a negative result, so V
        // is set. 0x50 - 0xB0 = 0xA0 with a borrow (carry clear).
        let cpu = run(vec![
            op(SEC, Implied),
            op(LDA, Immediate),
            0x50,
            op(SBC, Immediate),
            0xB0,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0xA0);
        assert!(cpu.status.overflow);
        assert!(!cpu.status.carry);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![
                op(SEC, Implied),
                op(LDA, Immediate),
                0x50,
                op(SBC, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x10)],
        );
        assert_eq!(cpu.register_a, 0x40);
    }
}

mod cmp {
    use super::*;

    #[test]
    fn sets_carry_when_greater() {
        // A(0x50) > M(0x30): carry set, result 0x20 is nonzero and positive
        let cpu = run(vec![
            op(LDA, Immediate),
            0x50,
            op(CMP, Immediate),
            0x30,
            op(BRK, Implied),
        ]);
        assert!(cpu.status.carry);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_carry_and_zero_when_equal() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x50,
            op(CMP, Immediate),
            0x50,
            op(BRK, Implied),
        ]);
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn clears_carry_when_less() {
        // A(0x30) < M(0x50): carry clear, 0x30 - 0x50 = 0xE0 sets negative
        let cpu = run(vec![
            op(LDA, Immediate),
            0x30,
            op(CMP, Immediate),
            0x50,
            op(BRK, Implied),
        ]);
        assert!(!cpu.status.carry);
        assert!(!cpu.status.zero);
        assert!(cpu.status.negative);
    }

    #[test]
    fn leaves_accumulator_unchanged() {
        let cpu = run(vec![
            op(LDA, Immediate),
            0x50,
            op(CMP, Immediate),
            0x30,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x50);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x50,
                op(CMP, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x50)],
        );
        assert!(cpu.status.zero);
    }
}

mod cpx {
    use super::*;

    #[test]
    fn sets_carry_when_greater() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x50,
            op(CPX, Immediate),
            0x30,
            op(BRK, Implied),
        ]);
        assert!(cpu.status.carry);
        assert!(!cpu.status.zero);
    }

    #[test]
    fn sets_zero_when_equal() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x50,
            op(CPX, Immediate),
            0x50,
            op(BRK, Implied),
        ]);
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
    }

    #[test]
    fn clears_carry_when_less() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x30,
            op(CPX, Immediate),
            0x50,
            op(BRK, Implied),
        ]);
        assert!(!cpu.status.carry);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x50,
                op(CPX, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x50)],
        );
        assert!(cpu.status.zero);
    }
}

mod cpy {
    use super::*;

    #[test]
    fn sets_carry_when_greater() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x50,
            op(CPY, Immediate),
            0x30,
            op(BRK, Implied),
        ]);
        assert!(cpu.status.carry);
        assert!(!cpu.status.zero);
    }

    #[test]
    fn sets_zero_when_equal() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x50,
            op(CPY, Immediate),
            0x50,
            op(BRK, Implied),
        ]);
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
    }

    #[test]
    fn clears_carry_when_less() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x30,
            op(CPY, Immediate),
            0x50,
            op(BRK, Implied),
        ]);
        assert!(!cpu.status.carry);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![
                op(LDY, Immediate),
                0x50,
                op(CPY, ZeroPage),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x50)],
        );
        assert!(cpu.status.zero);
    }
}

mod inc {
    use super::*;

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![op(INC, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x41)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x42);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn wraps_and_sets_zero_flag() {
        // 0xFF + 1 wraps to 0x00
        let cpu = run_seeded(
            vec![op(INC, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0xFF)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x00);
        assert!(cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_negative_flag() {
        // 0x7F + 1 = 0x80
        let cpu = run_seeded(
            vec![op(INC, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x7F)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x80);
        assert!(!cpu.status.zero);
        assert!(cpu.status.negative);
    }

    #[test]
    fn absolute() {
        let cpu = run_seeded(
            vec![op(INC, Absolute), 0x34, 0x12, op(BRK, Implied)],
            &[(0x1234, 0x41)],
        );
        assert_eq!(cpu.mem_read(0x1234), 0x42);
    }
}

mod dec {
    use super::*;

    #[test]
    fn zero_page() {
        let cpu = run_seeded(
            vec![op(DEC, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x43)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x42);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_zero_flag() {
        let cpu = run_seeded(
            vec![op(DEC, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x01)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x00);
        assert!(cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn wraps_and_sets_negative_flag() {
        // 0x00 - 1 wraps to 0xFF
        let cpu = run_seeded(
            vec![op(DEC, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x00)],
        );
        assert_eq!(cpu.mem_read(0x10), 0xFF);
        assert!(!cpu.status.zero);
        assert!(cpu.status.negative);
    }

    #[test]
    fn absolute() {
        let cpu = run_seeded(
            vec![op(DEC, Absolute), 0x34, 0x12, op(BRK, Implied)],
            &[(0x1234, 0x43)],
        );
        assert_eq!(cpu.mem_read(0x1234), 0x42);
    }
}

mod inx {
    use super::*;

    #[test]
    fn increments_x() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x41,
            op(INX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x42);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn wraps_and_sets_zero_flag() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0xFF,
            op(INX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x7F,
            op(INX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x80);
        assert!(cpu.status.negative);
    }
}

mod iny {
    use super::*;

    #[test]
    fn increments_y() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x41,
            op(INY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0x42);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn wraps_and_sets_zero_flag() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0xFF,
            op(INY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn sets_negative_flag() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x7F,
            op(INY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0x80);
        assert!(cpu.status.negative);
    }
}

mod dex {
    use super::*;

    #[test]
    fn decrements_x() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x43,
            op(DEX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x42);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_zero_flag() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x01,
            op(DEX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn wraps_and_sets_negative_flag() {
        let cpu = run(vec![
            op(LDX, Immediate),
            0x00,
            op(DEX, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0xFF);
        assert!(cpu.status.negative);
    }
}

mod dey {
    use super::*;

    #[test]
    fn decrements_y() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x43,
            op(DEY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0x42);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_zero_flag() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x01,
            op(DEY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0x00);
        assert!(cpu.status.zero);
    }

    #[test]
    fn wraps_and_sets_negative_flag() {
        let cpu = run(vec![
            op(LDY, Immediate),
            0x00,
            op(DEY, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_y, 0xFF);
        assert!(cpu.status.negative);
    }
}

mod sec {
    use super::*;

    #[test]
    fn sets_carry() {
        // status starts all-clear, so SEC alone proves the flag is set.
        let cpu = run(vec![op(SEC, Implied), op(BRK, Implied)]);
        assert!(cpu.status.carry);
    }
}

mod sed {
    use super::*;

    #[test]
    fn sets_decimal() {
        let cpu = run(vec![op(SED, Implied), op(BRK, Implied)]);
        assert!(cpu.status.decimal);
    }
}

mod sei {
    use super::*;

    #[test]
    fn sets_interrupt() {
        let cpu = run(vec![op(SEI, Implied), op(BRK, Implied)]);
        assert!(cpu.status.interrupt);
    }
}

mod clc {
    use super::*;

    #[test]
    fn clears_carry() {
        // SEC first, so the test proves CLC did the clearing.
        let cpu = run(vec![op(SEC, Implied), op(CLC, Implied), op(BRK, Implied)]);
        assert!(!cpu.status.carry);
    }
}

mod cld {
    use super::*;

    #[test]
    fn clears_decimal() {
        let cpu = run(vec![op(SED, Implied), op(CLD, Implied), op(BRK, Implied)]);
        assert!(!cpu.status.decimal);
    }
}

mod cli {
    use super::*;

    #[test]
    fn clears_interrupt() {
        let cpu = run(vec![op(SEI, Implied), op(CLI, Implied), op(BRK, Implied)]);
        assert!(!cpu.status.interrupt);
    }
}

mod clv {
    use super::*;

    #[test]
    fn clears_overflow() {
        // There is no set-overflow instruction, so seed V via PLP (pull 0xFF into
        // status) before CLV clears it.
        let cpu = run(vec![
            op(LDA, Immediate),
            0xFF,
            op(PHA, Implied),
            op(PLP, Implied), // V (and every other flag) now set
            op(CLV, Implied),
            op(BRK, Implied),
        ]);
        assert!(!cpu.status.overflow);
    }
}

mod nop {
    use super::*;

    #[test]
    fn does_not_alter_state() {
        // NOP between LDA and BRK must leave A and the flags untouched.
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(NOP, Implied),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x42);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn advances_one_byte() {
        // If NOP consumed the wrong number of bytes, the following LDA would
        // decode against the wrong operand. Reaching A = 0x42 proves it's a
        // single-byte no-op.
        let cpu = run(vec![
            op(NOP, Implied),
            op(LDA, Immediate),
            0x42,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x42);
    }
}

mod brk {
    use super::*;

    #[test]
    fn halts_execution() {
        // The LDA #$FF after BRK must never run, so A keeps the earlier 0x42.
        let cpu = run(vec![
            op(LDA, Immediate),
            0x42,
            op(BRK, Implied),
            op(LDA, Immediate),
            0xFF,
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x42);
    }
}

mod rti {
    use super::*;

    // RTI pulls status, then the return address (lo, hi). The stack is seeded by
    // pushing in the reverse of the pull order: hi, lo, then the status byte.

    #[test]
    fn returns_to_pulled_address() {
        // Push return address 0x0600, then a status byte, then RTI. Control should
        // resume at 0x0600, where a seeded `LDX #$99; BRK` proves we arrived.
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x06, // hi byte of return address
                op(PHA, Implied),
                op(LDA, Immediate),
                0x00, // lo byte of return address
                op(PHA, Implied),
                op(LDA, Immediate),
                0x00, // status byte (all flags clear)
                op(PHA, Implied),
                op(RTI, Implied),
            ],
            &[
                (0x0600, op(LDX, Immediate)),
                (0x0601, 0x99),
                (0x0602, op(BRK, Implied)),
            ],
        );
        assert_eq!(cpu.register_x, 0x99);
    }

    #[test]
    fn restores_status() {
        // Push a return address pointing straight at a BRK so nothing runs after
        // RTI to clobber the flags. Status byte 0xC3 = N V Z C set.
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x06,
                op(PHA, Implied),
                op(LDA, Immediate),
                0x00,
                op(PHA, Implied),
                op(LDA, Immediate),
                0xC3, // 1100_0011 -> negative, overflow, zero, carry
                op(PHA, Implied),
                op(RTI, Implied),
            ],
            &[(0x0600, op(BRK, Implied))],
        );
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
        assert!(cpu.status.overflow);
        assert!(cpu.status.negative);
        assert!(!cpu.status.interrupt);
        assert!(!cpu.status.decimal);
    }
}

// Each branch test uses `run_branch`, which sets a sentinel X = 0x42, applies
// the given flag setup, then branches over an `LDX #$FF`. A taken branch leaves
// X = 0x42; a non-taken branch falls through and leaves X = 0xFF. Flag setups:
// SEC/CLC and CLV act directly; zero/negative lean on LDA's side effects; V is
// seeded by pulling status byte 0x40 (overflow only) via PHA/PLP.

mod bcc {
    use super::*;

    #[test]
    fn taken_when_carry_clear() {
        assert_eq!(run_branch(&[op(CLC, Implied)], BCC).register_x, 0x42);
    }

    #[test]
    fn not_taken_when_carry_set() {
        assert_eq!(run_branch(&[op(SEC, Implied)], BCC).register_x, 0xFF);
    }
}

mod bcs {
    use super::*;

    #[test]
    fn taken_when_carry_set() {
        assert_eq!(run_branch(&[op(SEC, Implied)], BCS).register_x, 0x42);
    }

    #[test]
    fn not_taken_when_carry_clear() {
        assert_eq!(run_branch(&[op(CLC, Implied)], BCS).register_x, 0xFF);
    }
}

mod beq {
    use super::*;

    #[test]
    fn taken_when_zero_set() {
        assert_eq!(
            run_branch(&[op(LDA, Immediate), 0x00], BEQ).register_x,
            0x42
        );
    }

    #[test]
    fn not_taken_when_zero_clear() {
        assert_eq!(
            run_branch(&[op(LDA, Immediate), 0x01], BEQ).register_x,
            0xFF
        );
    }
}

mod bne {
    use super::*;

    #[test]
    fn taken_when_zero_clear() {
        assert_eq!(
            run_branch(&[op(LDA, Immediate), 0x01], BNE).register_x,
            0x42
        );
    }

    #[test]
    fn not_taken_when_zero_set() {
        assert_eq!(
            run_branch(&[op(LDA, Immediate), 0x00], BNE).register_x,
            0xFF
        );
    }

    #[test]
    fn branches_backward_with_negative_offset() {
        // LDX #$03; loop: DEX; BNE loop; BRK
        // The 0xFD (-3) offset jumps back to DEX, so X counts down to 0. This is
        // the case the `offset as i8` sign extension in `resolve` exists for.
        let cpu = run(vec![
            op(LDX, Immediate),
            0x03,
            op(DEX, Implied), // loop target
            op(BNE, Relative),
            0xFD, // -3 -> back to DEX
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.status.zero);
    }
}

mod bmi {
    use super::*;

    #[test]
    fn taken_when_negative_set() {
        assert_eq!(
            run_branch(&[op(LDA, Immediate), 0x80], BMI).register_x,
            0x42
        );
    }

    #[test]
    fn not_taken_when_negative_clear() {
        assert_eq!(
            run_branch(&[op(LDA, Immediate), 0x00], BMI).register_x,
            0xFF
        );
    }
}

mod bpl {
    use super::*;

    #[test]
    fn taken_when_negative_clear() {
        assert_eq!(
            run_branch(&[op(LDA, Immediate), 0x00], BPL).register_x,
            0x42
        );
    }

    #[test]
    fn not_taken_when_negative_set() {
        assert_eq!(
            run_branch(&[op(LDA, Immediate), 0x80], BPL).register_x,
            0xFF
        );
    }
}

mod bvc {
    use super::*;

    #[test]
    fn taken_when_overflow_clear() {
        assert_eq!(run_branch(&[op(CLV, Implied)], BVC).register_x, 0x42);
    }

    #[test]
    fn not_taken_when_overflow_set() {
        let setup = &[op(LDA, Immediate), 0x40, op(PHA, Implied), op(PLP, Implied)];
        assert_eq!(run_branch(setup, BVC).register_x, 0xFF);
    }
}

mod bvs {
    use super::*;

    #[test]
    fn taken_when_overflow_set() {
        let setup = &[op(LDA, Immediate), 0x40, op(PHA, Implied), op(PLP, Implied)];
        assert_eq!(run_branch(setup, BVS).register_x, 0x42);
    }

    #[test]
    fn not_taken_when_overflow_clear() {
        assert_eq!(run_branch(&[op(CLV, Implied)], BVS).register_x, 0xFF);
    }
}

mod jmp {
    use super::*;

    #[test]
    fn absolute() {
        // JMP $0600 skips the LDX #$FF and lands on seeded code that sets X.
        let cpu = run_seeded(
            vec![
                op(JMP, Absolute),
                0x00,
                0x06,
                op(LDX, Immediate),
                0xFF, // jumped over
                op(BRK, Implied),
            ],
            &[
                (0x0600, op(LDX, Immediate)),
                (0x0601, 0x42),
                (0x0602, op(BRK, Implied)),
            ],
        );
        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn indirect() {
        // JMP ($0600) reads the target from the pointer at 0x0600 (= 0x0700).
        let cpu = run_seeded(
            vec![op(JMP, Indirect), 0x00, 0x06, op(BRK, Implied)],
            &[
                (0x0600, 0x00), // pointer low
                (0x0601, 0x07), // pointer high -> target 0x0700
                (0x0700, op(LDX, Immediate)),
                (0x0701, 0x42),
                (0x0702, op(BRK, Implied)),
            ],
        );
        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn indirect_page_boundary_bug() {
        // With the pointer at 0x06FF, the 6502 reads the high byte from 0x0600
        // (same page) instead of 0x0700. Seed 0x0700 with a decoy that a
        // spec-correct CPU would use; this implementation must ignore it and
        // land on 0x0500.
        let cpu = run_seeded(
            vec![op(JMP, Indirect), 0xFF, 0x06, op(BRK, Implied)],
            &[
                (0x06FF, 0x00), // pointer low
                (0x0600, 0x05), // buggy high source -> target 0x0500
                (0x0700, 0xCC), // decoy high; must be ignored
                (0x0500, op(LDX, Immediate)),
                (0x0501, 0x42),
                (0x0502, op(BRK, Implied)),
            ],
        );
        assert_eq!(cpu.register_x, 0x42);
    }
}

mod jsr {
    use super::*;

    #[test]
    fn pushes_return_address_minus_one() {
        // JSR occupies 0x0000..=0x0002, so the return address is 0x0003 and the
        // pushed value is 0x0002 (hi at 0x01FD, lo at 0x01FC). The subroutine is
        // a bare BRK so nothing runs afterward to disturb the stack.
        let cpu = run_seeded(
            vec![op(JSR, Absolute), 0x00, 0x06, op(BRK, Implied)],
            &[(0x0600, op(BRK, Implied))],
        );
        assert_eq!(cpu.mem_read(0x01FD), 0x00); // return-1 high byte
        assert_eq!(cpu.mem_read(0x01FC), 0x02); // return-1 low byte
        assert_eq!(cpu.stack_pointer, 0xFB); // two bytes pushed from 0xFD
    }

    #[test]
    fn jumps_to_subroutine() {
        // The subroutine sets Y; reaching Y = 0x99 proves control transferred.
        let cpu = run_seeded(
            vec![op(JSR, Absolute), 0x00, 0x06, op(BRK, Implied)],
            &[
                (0x0600, op(LDY, Immediate)),
                (0x0601, 0x99),
                (0x0602, op(BRK, Implied)),
            ],
        );
        assert_eq!(cpu.register_y, 0x99);
    }

    #[test]
    fn round_trips_with_rts() {
        // JSR into a routine that sets Y and returns; execution must resume at
        // the LDX right after the 3-byte JSR. X = 0x42 proves RTS returned to the
        // correct address, Y = 0x99 proves the subroutine ran.
        let cpu = run_seeded(
            vec![
                op(JSR, Absolute),
                0x00,
                0x06,
                op(LDX, Immediate),
                0x42, // runs only if RTS returns here
                op(BRK, Implied),
            ],
            &[
                (0x0600, op(LDY, Immediate)),
                (0x0601, 0x99),
                (0x0602, op(RTS, Implied)),
            ],
        );
        assert_eq!(cpu.register_x, 0x42);
        assert_eq!(cpu.register_y, 0x99);
    }
}

mod rts {
    use super::*;

    #[test]
    fn returns_to_pulled_address_plus_one() {
        // Manually push 0x05FF (hi then lo, matching JSR's order). RTS pulls it
        // and adds 1, resuming at 0x0600 where seeded code sets X.
        let cpu = run_seeded(
            vec![
                op(LDA, Immediate),
                0x05, // return-1 high byte
                op(PHA, Implied),
                op(LDA, Immediate),
                0xFF, // return-1 low byte
                op(PHA, Implied),
                op(RTS, Implied),
            ],
            &[
                (0x0600, op(LDX, Immediate)),
                (0x0601, 0x42),
                (0x0602, op(BRK, Implied)),
            ],
        );
        assert_eq!(cpu.register_x, 0x42);
    }
}

mod asl {
    use super::*;

    #[test]
    fn accumulator() {
        // 0x01 << 1 = 0x02, old bit 7 was 0 so carry stays clear
        let cpu = run(vec![
            op(LDA, Immediate),
            0x01,
            op(ASL, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x02);
        assert!(!cpu.status.carry);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_carry_and_zero() {
        // 0x80 << 1 = 0x00, old bit 7 falls into carry
        let cpu = run(vec![
            op(LDA, Immediate),
            0x80,
            op(ASL, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_negative() {
        // 0x40 << 1 = 0x80
        let cpu = run(vec![
            op(LDA, Immediate),
            0x40,
            op(ASL, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(!cpu.status.carry);
        assert!(cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        // shifts memory in place: 0x02 << 1 = 0x04
        let cpu = run_seeded(
            vec![op(ASL, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x02)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x04);
    }
}

mod lsr {
    use super::*;

    #[test]
    fn accumulator() {
        // 0x02 >> 1 = 0x01, old bit 0 was 0 so carry stays clear
        let cpu = run(vec![
            op(LDA, Immediate),
            0x02,
            op(LSR, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x01);
        assert!(!cpu.status.carry);
        assert!(!cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn sets_carry_and_zero() {
        // 0x01 >> 1 = 0x00, old bit 0 falls into carry
        let cpu = run(vec![
            op(LDA, Immediate),
            0x01,
            op(LSR, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn always_clears_negative() {
        // LSR shifts a 0 into bit 7, so N is always cleared even from 0x80
        let cpu = run(vec![
            op(LDA, Immediate),
            0x80,
            op(LSR, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x40);
        assert!(!cpu.status.carry);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn zero_page() {
        // 0x04 >> 1 = 0x02
        let cpu = run_seeded(
            vec![op(LSR, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x04)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x02);
    }
}

mod rol {
    use super::*;

    #[test]
    fn rotates_carry_into_bit_0() {
        // carry set rotates a 1 into bit 0: 0x01 << 1 = 0x02, | 1 = 0x03
        let cpu = run(vec![
            op(SEC, Implied),
            op(LDA, Immediate),
            0x01,
            op(ROL, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x03);
        assert!(!cpu.status.carry); // old bit 7 of 0x01 was 0
    }

    #[test]
    fn clear_carry_leaves_bit_0_zero() {
        let cpu = run(vec![
            op(CLC, Implied),
            op(LDA, Immediate),
            0x01,
            op(ROL, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x02);
        assert!(!cpu.status.carry);
    }

    #[test]
    fn rotates_through_carry() {
        // old bit 7 leaves into carry while old carry enters bit 0:
        // 0x80 << 1 = 0x00, | carry(1) = 0x01, new carry = old bit 7 = 1
        let cpu = run(vec![
            op(SEC, Implied),
            op(LDA, Immediate),
            0x80,
            op(ROL, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x01);
        assert!(cpu.status.carry);
        assert!(!cpu.status.zero);
    }

    #[test]
    fn zero_page() {
        // carry clear: 0x01 << 1 = 0x02
        let cpu = run_seeded(
            vec![op(CLC, Implied), op(ROL, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x01)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x02);
    }
}

mod ror {
    use super::*;

    #[test]
    fn rotates_carry_into_bit_7() {
        // carry set rotates a 1 into bit 7: 0x00 >> 1 = 0x00, | 0x80 = 0x80
        let cpu = run(vec![
            op(SEC, Implied),
            op(LDA, Immediate),
            0x00,
            op(ROR, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(!cpu.status.carry); // old bit 0 of 0x00 was 0
        assert!(cpu.status.negative);
    }

    #[test]
    fn clear_carry_leaves_bit_7_zero() {
        // 0x02 >> 1 = 0x01, no carry in, no carry out
        let cpu = run(vec![
            op(CLC, Implied),
            op(LDA, Immediate),
            0x02,
            op(ROR, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x01);
        assert!(!cpu.status.carry);
        assert!(!cpu.status.negative);
    }

    #[test]
    fn rotates_out_carry_and_sets_zero() {
        // old bit 0 leaves into carry: 0x01 >> 1 = 0x00, carry in clear
        let cpu = run(vec![
            op(CLC, Implied),
            op(LDA, Immediate),
            0x01,
            op(ROR, Accumulator),
            op(BRK, Implied),
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.carry);
        assert!(cpu.status.zero);
    }

    #[test]
    fn zero_page() {
        // carry clear: 0x02 >> 1 = 0x01
        let cpu = run_seeded(
            vec![op(CLC, Implied), op(ROR, ZeroPage), 0x10, op(BRK, Implied)],
            &[(0x10, 0x02)],
        );
        assert_eq!(cpu.mem_read(0x10), 0x01);
    }
}
