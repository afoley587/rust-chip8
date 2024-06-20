use std::fs;
use std::fs::File;
use std::io::Read;
use rand;

const MEMORY_SIZE: usize = 4096;
const SPRITE_MEM_START: usize = 0x50;
const PROGRAM_MEM_START: usize = 0x200;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const KEYBOARD_SIZE: usize = 16;

const VIDEO_HEIGHT: usize = 32;
const VIDEO_WIDTH: usize = 64;

const KEYBOARD_SPRITES: [[u8; 5]; KEYBOARD_SIZE] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80]  // F
];

#[derive(Debug)]
pub struct Chip8 {
    registers: [u8; NUM_REGISTERS],
    memory: [u8; MEMORY_SIZE],
    index: u16,
    pc: usize,
    stack: [usize; STACK_SIZE],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    pub keyboard: [bool; KEYBOARD_SIZE],
    pub video: [u32; VIDEO_WIDTH * VIDEO_HEIGHT],
    opcode: usize,
    table: [fn(&mut Chip8); 0xF + 1],
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            registers: [0u8; NUM_REGISTERS],
            memory: [0u8; MEMORY_SIZE],
            index: 0,
            pc: PROGRAM_MEM_START,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keyboard: [false; KEYBOARD_SIZE],
            video: [0u32; VIDEO_WIDTH * VIDEO_HEIGHT],
            opcode: 0,
            table: [
                Chip8::table_0,
                Chip8::op_1nnn,
                Chip8::op_2nnn,
                Chip8::op_3xkk,
                Chip8::op_4xkk,
                Chip8::op_5xy0,
                Chip8::op_6xkk,
                Chip8::op_7xkk,
                Chip8::table_8,
                Chip8::op_9xy0,
                Chip8::op_annn,
                Chip8::op_bnnn,
                Chip8::op_cxkk,
                Chip8::op_dxyn,
                Chip8::table_e,
                Chip8::table_f,
            ],
        }
    }

    pub fn load_rom(filename: &str) -> Self {
        let mut f = File::open(&filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        let mut emulator = Self::new();

        let mut ctr = 0;
        for e in KEYBOARD_SPRITES.iter() {
            for &s in e.iter() {
                emulator.memory[SPRITE_MEM_START + ctr] = s;
                ctr += 1;
            }
        }

        for (i, &e) in buffer.iter().enumerate() {
            emulator.memory[PROGRAM_MEM_START + i] = e;
        }

        emulator.pc = PROGRAM_MEM_START;

        emulator
    }

    // Start OpCodes
    fn op_00e0(&mut self) {
        self.video = [0u32; VIDEO_WIDTH * VIDEO_HEIGHT];
    }

    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    fn op_1nnn(&mut self) {
        let address: usize = self.opcode & 0x0FFFusize;
        self.pc = address;
    }

    fn op_2nnn(&mut self) {
        let address: usize = self.opcode & 0x0FFFusize;
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = address;
    }

    fn op_3xkk(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let byte: u8 = self.opcode as u8 & 0x00FFu8;

        if self.registers[v_x] == byte {
            self.pc += 2;
        }
    }

    fn op_4xkk(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let byte: u8 = self.opcode as u8 & 0x00FFu8;

        if self.registers[v_x] != byte {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;

        if self.registers[v_x] == self.registers[v_y] {
            self.pc += 2;
        }
    }

    fn op_6xkk(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let byte: u8 = self.opcode as u8 & 0x00FFu8;

        self.registers[v_x] = byte;
    }

    fn op_7xkk(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let byte: u8 = self.opcode as u8 & 0x00FFu8;
        self.registers[v_x] = self.registers[v_x].wrapping_add(byte);
    }

    fn op_8xy0(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;
        self.registers[v_x] = self.registers[v_y];
    }

    fn op_8xy1(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;
        self.registers[v_x] |= self.registers[v_y];
    }

    fn op_8xy2(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;
        self.registers[v_x] &= self.registers[v_y];
    }

    fn op_8xy3(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;
        self.registers[v_x] ^= self.registers[v_y];
    }

    fn op_8xy4(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;

        let (sum, carry) = self.registers[v_x].overflowing_add(self.registers[v_y]);
        self.registers[0xF] = carry as u8;
        self.registers[v_x] = sum;
    }

    fn op_8xy5(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;

        let (diff, borrow) = self.registers[v_x].overflowing_sub(self.registers[v_y]);
        self.registers[0xF] = !borrow as u8;
        self.registers[v_x] = diff;
    }

    fn op_8xy6(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        self.registers[0xF] = self.registers[v_x] & 0x1u8;
        self.registers[v_x] >>= 1;
    }

    fn op_8xy7(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;

        let (diff, borrow) = self.registers[v_y].overflowing_sub(self.registers[v_x]);
        self.registers[0xF] = !borrow as u8;
        self.registers[v_x] = diff;
    }

    fn op_8xye(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        self.registers[0xF] = (self.registers[v_x] & 0x80u8) >> 7u8;
        self.registers[v_x] <<= 1;
    }

    fn op_9xy0(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;

        if self.registers[v_x] != self.registers[v_y] {
            self.pc += 2;
        }
    }

    fn op_annn(&mut self) {
        let address: u16 = self.opcode as u16 & 0x0FFFu16;
        self.index = address;
    }

    fn op_bnnn(&mut self) {
        let address: u16 = self.opcode as u16 & 0x0FFFu16;
        self.pc = self.registers[0] as usize + address as usize;
    }

    fn op_cxkk(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let byte = (self.opcode & 0x00FFusize) as u8;
        self.registers[v_x] = byte & rand::random::<u8>();
    }

    fn op_dxyn(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let v_y: usize = (self.opcode & 0x00F0usize) >> 4usize;
        let height: usize = self.opcode & 0x000Fusize;

        let x_pos: usize = self.registers[v_x] as usize % VIDEO_WIDTH;
        let y_pos: usize = self.registers[v_y] as usize % VIDEO_HEIGHT;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.memory[self.index as usize + row];

            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0x80u8 >> col);
                let video_pos = ((y_pos + row) * VIDEO_WIDTH + (x_pos + col)) % (VIDEO_WIDTH * VIDEO_HEIGHT);
                let screen_pixel = self.video[video_pos];

                if sprite_pixel != 0 {
                    if screen_pixel == 0xFFFFFFFFu32 {
                        self.registers[0xF] = 1;
                    }
                    self.video[video_pos] ^= 0xFFFFFFFFu32;
                }
            }
        }
    }

    fn op_ex9e(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let key: u8 = self.registers[v_x];

        if self.keyboard[key as usize] {
            self.pc += 2;
        }
    }

    fn op_exa1(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let key: u8 = self.registers[v_x];

        if !self.keyboard[key as usize] {
            self.pc += 2;
        }
    }

    fn op_fx07(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        self.registers[v_x] = self.delay_timer;
    }

    fn op_fx0a(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;

        if let Some(i) = self.keyboard.iter().position(|&k| k) {
            self.registers[v_x] = i as u8;
        } else {
            self.pc -= 2;
        }
    }

    fn op_fx15(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        self.delay_timer = self.registers[v_x];
    }

    fn op_fx18(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        self.sound_timer = self.registers[v_x];
    }

    fn op_fx1e(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        self.index += self.registers[v_x] as u16;
    }

    fn op_fx29(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let val: u8 = self.registers[v_x];
        self.index = (SPRITE_MEM_START + (5 * val) as usize) as u16;
    }

    fn op_fx33(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        let val: u8 = self.registers[v_x];

        self.memory[self.index as usize + 2] = val % 10;
        self.memory[self.index as usize + 1] = (val / 10) % 10;
        self.memory[self.index as usize] = (val / 100) % 10;
    }

    fn op_fx55(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        
        for i in 0..=v_x {
            self.memory[self.index as usize + i] = self.registers[i];
        }
    }

    fn op_fx65(&mut self) {
        let v_x: usize = (self.opcode & 0x0F00usize) >> 8usize;
        
        for i in 0..=v_x {
            self.registers[i] = self.memory[self.index as usize + i];
        }
    }

    fn op_null(&mut self) {}

    pub fn cycle(&mut self) {
        self.opcode = ((self.memory[self.pc] as usize) << 8) | (self.memory[self.pc + 1] as usize);
        self.pc += 2;

        match (self.opcode & 0xF000) >> 12 {
            0x0 => self.table_0(),
            0x8 => self.table_8(),
            0xE => self.table_e(),
            0xF => self.table_f(),
            _ => (self.table[(self.opcode & 0xF000usize) >> 12usize])(self),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn table_0(&mut self) {
        let mut table_0: [fn(&mut Chip8); 0xE + 1] = [
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
        ];
        table_0[0x0] = Chip8::op_00e0;
        table_0[0xE] = Chip8::op_00ee;

        table_0[self.opcode & 0x000Fusize](self);
    }

    fn table_8(&mut self) {
        let mut table_8: [fn(&mut Chip8); 0xE + 1] = [
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
        ];

        table_8[0x0] = Chip8::op_8xy0;
        table_8[0x1] = Chip8::op_8xy1;
        table_8[0x2] = Chip8::op_8xy2;
        table_8[0x3] = Chip8::op_8xy3;
        table_8[0x4] = Chip8::op_8xy4;
        table_8[0x5] = Chip8::op_8xy5;
        table_8[0x6] = Chip8::op_8xy6;
        table_8[0x7] = Chip8::op_8xy7;
        table_8[0xE] = Chip8::op_8xye;

        table_8[self.opcode & 0x000Fusize](self);
    }

    fn table_e(&mut self) {
        let mut table_e: [fn(&mut Chip8); 0xE + 1] = [
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
        ];
        table_e[0x1] = Chip8::op_exa1;
        table_e[0xE] = Chip8::op_ex9e;

        table_e[self.opcode & 0x000Fusize](self);
    }

    fn table_f(&mut self) {
        let mut table_f: [fn(&mut Chip8); 0x65 + 1] = [
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
            Chip8::op_null,
        ];

        table_f[0x07] = Chip8::op_fx07;
        table_f[0x0A] = Chip8::op_fx0a;
        table_f[0x15] = Chip8::op_fx15;
        table_f[0x18] = Chip8::op_fx18;
        table_f[0x1E] = Chip8::op_fx1e;
        table_f[0x29] = Chip8::op_fx29;
        table_f[0x33] = Chip8::op_fx33;
        table_f[0x55] = Chip8::op_fx55;
        table_f[0x65] = Chip8::op_fx65;

        table_f[self.opcode & 0x00FFusize](self);
    }
}
