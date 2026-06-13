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
    for &(addr, value) in seed {
        cpu.mem_write(addr, value);
    }
    cpu.run();
    cpu
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
        // base = 0xFF, X = 2 -> wraps to 0x01 (not 0x101)
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x02,
                op(LDA, ZeroPageX),
                0xFF,
                op(BRK, Implied),
            ],
            &[(0x01, 0x42)],
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
        // X = 1, zp operand 0x10 -> pointer read from 0x11/0x12 = 0x3000
        let cpu = run_seeded(
            vec![
                op(LDX, Immediate),
                0x01,
                op(LDA, IndirectX),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x11, 0x00), (0x12, 0x30), (0x3000, 0x42)],
        );
        assert_eq!(cpu.register_a, 0x42);
    }

    #[test]
    fn indirect_y() {
        // pointer at 0x10/0x11 = 0x3000, then + Y(4) = 0x3004
        let cpu = run_seeded(
            vec![
                op(LDY, Immediate),
                0x04,
                op(LDA, IndirectY),
                0x10,
                op(BRK, Implied),
            ],
            &[(0x10, 0x00), (0x11, 0x30), (0x3004, 0x42)],
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
        // pointer at 0x11/0x12 = 0x3000
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
            &[(0x11, 0x00), (0x12, 0x30)],
        );
        assert_eq!(cpu.mem_read(0x3000), 0x42);
    }

    #[test]
    fn indirect_y() {
        // pointer at 0x10/0x11 = 0x3000, + Y(4) = 0x3004
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
            &[(0x10, 0x00), (0x11, 0x30)],
        );
        assert_eq!(cpu.mem_read(0x3004), 0x42);
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
