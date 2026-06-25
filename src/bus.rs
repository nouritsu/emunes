const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU_START: u16 = 0x2000;
const PPU_END: u16 = 0x3FFF;

const RESET_VECTOR_START: u16 = 0xFFFC;
const RESET_VECTOR_END: u16 = 0xFFFD;

#[derive(Debug)]
pub struct Bus {
    cpu_vram: [u8; 2048],
    reset_vector: u16,
}

pub trait Mem {
    // byte
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);

    // word
    fn mem_read_u16(&self, addr: u16) -> u16;
    fn mem_write_u16(&mut self, addr: u16, data: u16);
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            cpu_vram: [0; 2048],
            reset_vector: 0,
        }
    }
}

impl Mem for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }

            PPU_START..=PPU_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not implemented")
            }

            RESET_VECTOR_START..=RESET_VECTOR_END => {
                self.reset_vector.to_le_bytes()[(addr - RESET_VECTOR_START) as usize]
            }

            _ => {
                println!("ignoring memory access at 0x{addr:x}");
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }

            PPU_START..=PPU_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not implemented")
            }

            RESET_VECTOR_START..=RESET_VECTOR_END => {
                let mut bytes = self.reset_vector.to_le_bytes();
                bytes[(addr - RESET_VECTOR_START) as usize] = data;
                self.reset_vector = u16::from_le_bytes(bytes);
            }

            _ => {
                println!("ignoring memory write at 0x{addr:x}");
            }
        }
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr.wrapping_add(1)) as u16;

        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let lo = (data & 0xff) as u8;
        let hi = (data >> 8) as u8;

        self.mem_write(addr, lo);
        self.mem_write(addr.wrapping_add(1), hi);
    }
}
