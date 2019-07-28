use rand::Rng;

struct Font {
    address: u16,
    value: [u8; 5]
}

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const fonts: [Font; 16] = [
            Font{address: 0, value: [0xF0, 0x90, 0x90, 0x90, 0xF0]},
            Font{address: 5, value: [0x20, 0x60, 0x20, 0x20, 0x70]},
            Font{address: 10, value: [0xF0, 0x10, 0xF0, 0x80, 0xF0]},
            Font{address: 15, value: [0xF0, 0x10, 0xF0, 0x10, 0xF0]},
            Font{address: 20, value: [0x90, 0x90, 0xF0, 0x10, 0x10]},
            Font{address: 25, value: [0xF0, 0x80, 0xF0, 0x10, 0xF0]},
            Font{address: 30, value: [0xF0, 0x80, 0xF0, 0x90, 0xF0]},
            Font{address: 35, value: [0xF0, 0x10, 0x20, 0x40, 0x40]},
            Font{address: 40, value: [0xF0, 0x90, 0xF0, 0x90, 0xF0]},
            Font{address: 45, value: [0xF0, 0x90, 0xF0, 0x10, 0xF0]},
            Font{address: 50, value: [0xF0, 0x90, 0xF0, 0x90, 0x90]},
            Font{address: 55, value: [0xE0, 0x90, 0xE0, 0x90, 0xE0]},
            Font{address: 60, value: [0xF0, 0x80, 0x80, 0x80, 0xF0]},
            Font{address: 65, value: [0xE0, 0x90, 0x90, 0x90, 0xE0]},
            Font{address: 70, value: [0xF0, 0x80, 0xF0 ,0x80, 0xF0]},
            Font{address: 75, value: [0xF0, 0x80, 0xF0, 0x80, 0x80]}
        ];

pub struct Chip8 {
    pc: usize,
    memory: [u8; 4092],
    registers: [u8; 16],
    pub screen: [[u8; 64]; 32],
    pub keys: [u8; 16],
    I: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<usize>,
}
impl Chip8 {
    pub fn init() -> Self {
        let mut memory = [0; 4092];
        let mut index = 0;
        for font in fonts.iter() {
            for line in font.value.iter() {
                memory[index] = *line;
                index += 1;
            }
        }

        return Chip8 {
        pc: 0x200,
        registers: [0; 16],
        memory: memory,
        screen: [[0; 64]; 32],
        keys: [0; 16],
        I: 0,
        delay_timer: 0,
        sound_timer: 0,
        stack: Vec::new()
        };
    }

    fn fetch_instruction(&self, pc: usize) -> u16 {
        return (self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16;
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.memory[0x200 ..].copy_from_slice(&data[..(4092 - 0x200)]);
    }

    pub fn emulate_cycle(&mut self) {
        let opcode: u16 = self.fetch_instruction(self.pc);
        self.execute_instruction(opcode);
    }

    pub fn tick_delay_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    pub fn tick_sound_timer(&mut self) {
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn execute_instruction(&mut self, opcode: u16) {

        let parts = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8
        );

        match parts {
            (0, 0, 0xE, 0) => {
                for pixel in self.screen.iter_mut().flat_map(|y| y.iter_mut()) {
                    *pixel = 0;
                }
                self.pc += 2;
            },
            (0, 0, 0xE, 0xE) => {
                let result = self.stack.pop();
                match result {
                    Some(pc) => self.pc = pc,
                    None => panic!("Cannot return from subroutine")
                }
                self.pc += 2;
            },
            (0x1, _, _, _) => {
                let address = (parts.1 as u16) << 8 | (parts.2 as u16) << 4 | parts.3 as u16;
                self.pc = address as usize;
            },
            (0x2, _, _, _) => {
                // call subroutine
                let address = (parts.1 as u16) << 8 | (parts.2 as u16) << 4 | parts.3 as u16;
                let opcode = self.fetch_instruction(address as usize);
                self.stack.push(self.pc);
                self.pc = address as usize;
                self.execute_instruction(opcode);
            },
            (0x3, _, _, _) => {
                let register = parts.1 as usize;
                let value = parts.2 << 4 | parts.3;
                if self.registers[register] == value {
                    self.pc += 2;
                }
                self.pc +=2;
            },
            (0x4, _, _, _) => {
                let register = parts.1 as usize;
                let value = parts.2 << 4 | parts.3;

                if self.registers[register] != value {
                    self.pc += 2;
                }
                self.pc +=2;
            },
            (0x6, _, _, _) => {
                let register = parts.1 as usize;
                let value = parts.2 << 4 | parts.3;
                self.registers[register] = value;
                self.pc += 2;
            },
            (0x7, _, _, _) => {
                let register = parts.1 as usize;
                let value = parts.2 << 4 | parts.3;
                let mut result: u16 = self.registers[register] as u16 + value as u16;
                if result > 255 {
                    result = result % 256;
                }
                self.registers[register] = result as u8;
                self.pc +=2;
            },
            (0x8, _, _, 0) => {
                let x = parts.1 as usize;
                let y = parts.2 as usize;
                self.registers[x] = self.registers[y];
                self.pc += 2;
            },
            (0x8, _, _, 1) => {
                let x = parts.1 as usize;
                let y = parts.2 as usize;
                self.registers[x] = self.registers[x] | self.registers[y];
                self.pc += 2;
            },
            (0x8, _, _, 2) => {
                let x = parts.1 as usize;
                let y = parts.2 as usize;
                self.registers[x] = self.registers[x] & self.registers[y];
                self.pc += 2;
            },
            (0x8, _, _, 3) => {
                let x = parts.1 as usize;
                let y = parts.2 as usize;
                self.registers[x] = self.registers[x] ^ self.registers[y];
                self.pc += 2;
            },
            (0x8, _, _, 4) => {
                let vx = parts.1 as usize;
                let vy = parts.2 as usize;

                let result: u16 = self.registers[vx] as u16 + self.registers[vy] as u16;
                if result > 255 {
                    self.registers[vx] = (result % 256) as u8;
                    self.registers[0xF] = 1;
                } else {
                    self.registers[vx] = result as u8;
                    self.registers[0xF] = 0;
                }
                self.pc += 2;
            },
            (0x8, _, _, 5) => {
                let x = parts.1 as usize;
                let y = parts.2 as usize;

                let vx = self.registers[x];
                let vy = self.registers[y];

                self.registers[x] = ((vx - vy) as u16 % 256) as u8;

                if vy > vx {
                    self.registers[0xF] = 0;
                } else {
                    self.registers[0xF] = 1;
                }

                self.pc += 2;
            },
            (0x8, _, _, 6) => {
                let x = parts.1 as usize;
                self.registers[0xF] = self.registers[x] & 0x1;
                self.registers[x] = self.registers[x] >> 1;
                self.pc += 2;
            },
            (0xA, _, _, _) => {
                let address = (parts.1 as u16) << 8 | (parts.2 as u16) << 4 | parts.3 as u16;
                self.I = address as u16;
                self.pc +=2;
            },
            (0xC, _, _, _) => {
                let register = parts.1 as usize;
                let value = parts.2 << 4 | parts.3;
                let random_value = rand::thread_rng().gen_range(0, 255);
                self.registers[register] = random_value & value;
                self.pc += 2;
            },
            (0xD, _, _, _,) => {
                let x = self.registers[parts.1 as usize] as usize;
                let y = self.registers[parts.2 as usize] as usize;
                let height = parts.3;

                let mut temporary_address_pointer = self.I as usize;
                let mut yy = y;

                let mut pixel_from_set_to_unset: u8 = 0;
                let mut checked: bool = false;

                for _ in 0..height {
                    let line = self.memory[temporary_address_pointer];
                    let mut xx = x;
                    for bit in (0..8).rev() {
                        let pixel = (line & (2 as u8).pow(bit)) >> bit;
                        // MOD because if the sprite is outside the screen it wraps around the other side
                        let screen_x = xx % WIDTH;
                        let screen_y = yy % HEIGHT;
                        let old_pixel_value = self.screen[screen_y][screen_x];
                        self.screen[screen_y][screen_x] = old_pixel_value ^ pixel;
                        if !checked && self.screen[screen_y][screen_x] == 0 && old_pixel_value == 1 {
                            checked = true;
                            pixel_from_set_to_unset = 1;
                        }
                        xx +=1;
                    }
                    yy += 1;
                    temporary_address_pointer +=1;
                }
                self.registers[0xF] = pixel_from_set_to_unset;
                self.pc +=2;
            },
            (0xE, _, 9, 0xE) => {
                let register = parts.1 as usize;
                let key = self.registers[register] as usize;
                if self.keys[key] == 1 {
                    self.pc +=2;
                }
                self.pc += 2;
            }
            (0xE, _, 0xA, 1) => {
                let register = parts.1 as usize;
                let key = self.registers[register] as usize;
                if self.keys[key] == 0 {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            (0xF, _, 0, 7) => {
                let register = parts.1 as usize;
                self.registers[register] = self.delay_timer;
                self.pc += 2;
            },
            (0xF, _, 0, 0xA) => {
                let register = parts.1 as usize;
                for (i, pressed) in self.keys.iter().enumerate() {
                    if *pressed == 1 {
                        self.registers[register] = i as u8;
                        self.pc +=2;
                        break;
                    }
                }
            },
            (0xF, _, 1, 5) => {
                let register = parts.1 as usize;
                self.delay_timer = self.registers[register];
                self.pc += 2;
            },
            (0xF, _, 1, 8) => {
                let register = parts.1 as usize;
                self.sound_timer = self.registers[register];
                self.pc += 2;
            },
            (0xF, _, 1, 0xE) => {
                let register = parts.1 as usize;
                self.I += self.registers[register] as u16;
                self.pc += 2;
            }
            (0xF, _, 2, 9) => {
                let register = parts.1 as usize;
                let value = self.registers[register] as usize;
                self.I = fonts[value].address;
                self.pc += 2;
            },
            (0xF, _, 3, 3) => {
                let register = parts.1 as usize;
                let value = self.registers[register];
                let hundreds = value / 100;
                let tens = value / 10 % 10;
                let digit = value % 10;
                let address = self.I as usize;
                self.memory[address] = hundreds;
                self.memory[address + 1] = tens;
                self.memory[address + 2] = digit;

                self.pc +=2;
            },
            (0xF, _, 6, 5) => {
                let register = parts.1 as usize;
                let mut offset = self.I as usize;
                let mut i = 0;
                while i <= register {
                    self.registers[i] = self.memory[offset];
                    offset += 1;
                    i += 1;
                }
                
                self.pc += 2;
            },
            _ => panic!("Opcode {:X?} not implemented yet", opcode)
        }
    }
}
