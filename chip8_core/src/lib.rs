use rand::Rng;

const REGS_NUM: usize = 16;
const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const KEYS_NUM: usize = 16;

const START_MEMORY_ADDR: u16 = 0x200;
const FONTS_SIZE: usize = 80;

pub const CLOCK_SPEED: u64 = 600;


// Core emulator for Chip 8 system
pub struct Emulator {
    pc: u16,
    ram: [u8; RAM_SIZE],
    v_reg: [u8; REGS_NUM],
    i_reg: u16,

    sp: u16,
    stack: [u16; STACK_SIZE],

    delay_timer: u8,
    sound_timer: u8,

    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keys: [bool; KEYS_NUM],
}

impl Emulator {
    // Create a new emulator
    pub fn new() -> Self {
        let mut emu = Self {
            pc: START_MEMORY_ADDR,
            ram: [0; RAM_SIZE],
            v_reg: [0; REGS_NUM],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            keys: [false; KEYS_NUM],
        };

        emu.ram[..FONTS_SIZE].copy_from_slice(&FONT_SET);
        emu
    }

    // Reset console's state
    pub fn reset(&mut self) {
        self.pc = START_MEMORY_ADDR;
        self.ram = [0; RAM_SIZE];
        self.v_reg = [0; REGS_NUM];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.keys = [false; KEYS_NUM];

        self.ram[..FONTS_SIZE].copy_from_slice(&FONT_SET);
    }

    // Run one cycle of emulator
    pub fn tick(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute(opcode)
    }

    // Execute current opcode
    fn execute(&mut self, opcode: u16) {
        let part1: u16 = (opcode & 0xF000) >> 12;
        let part2: u16 = (opcode & 0x0F00) >> 8;
        let part3: u16 = (opcode & 0x00F0) >> 4;
        let part4: u16 = opcode & 0x000F;

        match (part1, part2, part3, part4) {
            // NOP - literally nothing
            (0, 0, 0, 0) => return,
            // CLS - clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // RET - return from subroutine
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop();
            }
            // JP, addr - jump to the given address
            (0x1, _, _, _) => {
                self.pc = opcode & 0xFFF;
            }
            // CALL, addr - call given subroutine
            (0x2, _, _, _) => {
                self.push(self.pc);
                self.pc = opcode & 0xFFF;
            }
            // SE Vx, byte - skip next instruction if Vx == kk
            (0x3, _, _, _) => {
                if self.v_reg[part2 as usize] == (opcode & 0xFF) as u8 {
                    self.pc += 2;
                }
            }
            // SNE Vx, byte - skip next instruction if Vx != kk
            (0x4, _, _, _) => {
                if self.v_reg[part2 as usize] != (opcode & 0xFF) as u8 {
                    self.pc += 2;
                }
            }
            // SE Vx, Vy - skip next instruction if Vx == Vy
            (0x5, _, _, 0) => {
                if self.v_reg[part2 as usize] == self.v_reg[part3 as usize] {
                    self.pc += 2;
                }
            }
            // LD Vx, byte - set Vx to kk
            (0x6, _, _, _) => {
                self.v_reg[part2 as usize] = (opcode & 0xFF) as u8;
            }
            // ADD Vx, byte - Add kk to Vx
            (0x7, _, _, _) => {
                let x: usize = part2 as usize;
                self.v_reg[x] = self.v_reg[x].wrapping_add((opcode & 0xFF) as u8);
            }
            // LD Vx, Vy - Set Vx to Vy
            (0x8, _, _, 0) => {
                self.v_reg[part2 as usize] = self.v_reg[part3 as usize];
            }
            // OR Vx, Vy - perform OR on Vx, Vy, save result in Vx
            (0x8, _, _, 1) => {
                self.v_reg[part2 as usize] |= self.v_reg[part3 as usize];
            }
            // AND Vx, Vy - save result of AND operation between Vx and Vy in Vx
            (0x8, _, _, 2) => {
                self.v_reg[part2 as usize] &= self.v_reg[part3 as usize];
            }
            // XOR Vx, Vy - save result of XOR operation of Vx and Vy in Vx
            (0x8, _, _, 3) => {
                self.v_reg[part2 as usize] ^= self.v_reg[part3 as usize];
            }
            // ADD Vx, Vy - add Vx to Vy, set VF to 1 if there overflowed, save reminder in Vx
            (0x8, _, _, 4) => {
                let x = part2 as usize;
                let (new_vx, overflow) = self.v_reg[x].overflowing_add(
                    self.v_reg[part3 as usize]
                );

                self.v_reg[x] = new_vx;
                self.v_reg[0xFusize] = if overflow { 1 } else { 0 };
            }
            // SUB Vx, Vy - subtract Vy from Vx, set VF to 1 if Vx > Vy
            (0x8, _, _, 5) => {
                let x = part2 as usize;
                let (new_vx, overflow) = self.v_reg[x].overflowing_sub(
                    self.v_reg[part3 as usize]
                );

                self.v_reg[x] = new_vx;
                self.v_reg[0xFusize] = if overflow { 0 } else { 1 };
            }
            // SHR Vx - set VF to the least significant bit of Vx, divide Vx by 2
            (0x8, _, _, 6) => {
                let x = part2 as usize;
                self.v_reg[0xFusize] = self.v_reg[x] &  1;
                self.v_reg[x] >>= 1;
            }
            // SUBN Vx, Vy - Set Vx to Vy - Vx, if there was an borrow, set VF to 1
            (0x8, _, _, 7) => {
                let x = part2 as usize;
                let (new_vx, overflow) = self.v_reg[part3 as usize].overflowing_sub(
                    self.v_reg[x]
                );

                self.v_reg[x] = new_vx;
                self.v_reg[0xFusize] = if overflow { 1 } else { 0 };
            }
            // SHL Vx - Set VF to the most significant bit of Vx, multiply Vx by two
            (0x8, _, _, 0xE) => {
                let x = part2 as usize;

                self.v_reg[0xFusize] = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
            }
            // SNE Vx, Vy - Skip next instruction if Vx != Vy
            (0x9, _, _, 0) => {
                if self.v_reg[part2 as usize] != self.v_reg[part3 as usize] {
                    self.pc += 2;
                }
            }
            // LD I, addr - set value of register I to the given address
            (0xA, _, _, _) => {
                self.i_reg = opcode & 0x0FFF;
            }
            // JP V0, addr - jump to location addr + V0
            (0xB, _, _, _) => {
                self.pc = (opcode & 0x0FFF) + self.v_reg[0] as u16;
            }
            // RND Vx, byte - store result of random number & kk in Vx
            (0xC, _, _, _) => {
                let rnd: u8 = rand::thread_rng().gen();
                self.v_reg[part2 as usize] = rnd & (opcode & 0x0FF) as u8;
            }
            // DRW Vx, Vy, nibble - display n-byte sprite at memory location I, position (Vx, Vy)
            // set VF to collision
            (0xD, _, _, _) => {
                let x_coord: u16 = self.v_reg[part2 as usize] as u16;
                let y_coord: u16 = self.v_reg[part3 as usize] as u16;

                let mut flipped: bool = false;

                for y_line in 0..part4 {
                    let pixels = self.ram[(self.i_reg + y_line) as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b10000000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let index = x + y * SCREEN_WIDTH;
                            flipped |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xFusize] = 1;
                }
                else {
                    self.v_reg[0xFusize] = 0;
                }

            }
            // SKP Vx - skip next instruction if key with an index of Vx is pressed
            (0xE, _, 9, 0xE) => {
                if self.keys[self.v_reg[part2 as usize] as usize] {
                    self.pc += 2;
                }
            }
            // SKNP Vx - skip next instruction if key with an index of Vx isn't pressed
            (0xE, _, 0xA, 1) => {
                if !self.keys[self.v_reg[part2 as usize] as usize] {
                    self.pc += 2;
                }
            }
            // LD Vx, DT - set Vx to delay timer's value
            (0xF, _, 0, 7) => {
                self.v_reg[part2 as usize] = self.delay_timer;
            }
            // LD Vx, K - wait for a key press then store the value of it into Vx
            (0xF, _, 0, 0xA) => {
                for key_num in 0..KEYS_NUM {
                    if self.keys[key_num] {
                        self.v_reg[part2 as usize] = key_num as u8;
                        return;
                    }
                }

                self.pc -= 2;
            }
            // LD DT, Vx - set delay timer to Vx
            (0xF, _, 1, 5) => {
                self.delay_timer = self.v_reg[part2 as usize];
            }
            // LD ST, Vx - set sound timer to Vx
            (0xF, _, 1, 8) => {
                self.sound_timer = self.v_reg[part2 as usize];
            }
            // ADD I, Vx - set I to I + Vx
            (0xF, _, 1, 0xE) => {
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[part2 as usize] as u16);
            }
            // LD F, Vx - set I to location of sprite for digit Vx
            (0xF, _, 2, 9) => {
                self.i_reg = self.v_reg[part2 as usize] as u16 * 5;
            }
            // LD B, Vx - store binary decimal of a number in memory, starting at location I
            (0xF, _, 3, 3) => {
                let vx = self.v_reg[part2 as usize] as f32;

                self.ram[self.i_reg as usize] = (vx / 100.0).floor() as u8;
                self.ram[(self.i_reg + 1) as usize] = ((vx / 10.0) % 10.0).floor() as u8;
                self.ram[(self.i_reg + 2) as usize] = (vx % 10.0) as u8;
            }
            // LD [I], Vx - store registers V0 to Vx in memory starting at location I
            (0xF, _, 5, 5) => {
                for num in 0..=part2 {
                    self.ram[(self.i_reg + num) as usize] = self.v_reg[num as usize];
                }
            }
            // LD Vx, [I] - read register from V0 to Vx staring at memory location I
            (0xF, _, 6, 5) => {
                for num in 0..=part2 {
                    self.v_reg[num as usize] = self.ram[(self.i_reg + num) as usize];
                }
            }
            // Error safeguard
            (_, _, _, _) => unimplemented!("Opcode {} is unimplemented", opcode),
        }
    }

    // Detect next opcode
    fn fetch_opcode(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        self.pc += 2;
        (higher_byte << 8) | lower_byte
    }

    // Decrease time, handle timing out
    pub fn time_tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // TODO: PLAY SOUND
            }
            self.sound_timer -= 1;
        }
    }

    // Push value onto stack
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
    // Pop value out of stack
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // Get access to console's display
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    // Set key states
    pub fn keypress(&mut self, index: usize, pressed: bool) {
        self.keys[index] = pressed;
    }

    // Load data to RAM
    pub fn load_data(&mut self, data: &[u8]) {
        let end: usize = (START_MEMORY_ADDR + data.len() as u16) as usize;
        self.ram[START_MEMORY_ADDR as usize..end].copy_from_slice(data);
    }
}

// Often used sprites
const FONT_SET: [u8; FONTS_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,  // 0
    0x20, 0x60, 0x20, 0x20, 0x70,  // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,  // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,  // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,  // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,  // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,  // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,  // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,  // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,  // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,  // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,  // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,  // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,  // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,  // E
    0xF0, 0x80, 0xF0, 0x80, 0x80,  // F
];