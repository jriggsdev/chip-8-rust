use rand::{Rng};

/// The frame buffer's width in pixels
pub const DISPLAY_WIDTH: usize = 64;

/// The frame buffer's height in pixels
pub const DISPLAY_HEIGHT: usize = 32;

/// The address at which to start loading program bytes
const PROGRAM_START_ADDRESS: u16 = 0x200;

/// The number bytes of memory
const MEMORY_SIZE: usize = 4096;

/// The number of bytes the stack can hold
const STACK_SIZE: usize = 16;

/// The number of variable registers
const VARIABLE_REGISTER_COUNT: usize = 16;

/// The number of keys on the keypad
const NUM_KEYS: usize = 16;

/// The address at which to start loading the font
const FONT_START_ADDRESS: usize = 0x50;

/// The font sprite data consisting of hexadecimal numbers 0-F
const FONT: [u8; 16 * 5] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

/// Specifies an emulator type to run the program as.
/// Emulator type affects how certain instructions are interpreted depending on the program it
/// may work on one type and not the other.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EmulatorType {
    /// Tells the Chip-8 interpreter to interpret instructions as the COSMAC-VIP would
    CosmacVip,
    /// Tells the Chip-8 interpreter to interpret instructions as the CHIP-48 would
    Chip48
}

/// Represents a key on the Chip-8 keypad.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Chip8Key {
    // The '0' key
    Zero,
    /// The '1' key
    One,
    /// The '2' key
    Two,
    /// The '3' key
    Three,
    /// The '4' key
    Four,
    /// The '5' key
    Five,
    /// The '6' key
    Six,
    /// The '7' key
    Seven,
    /// The '8' key
    Eight,
    /// The '9' key
    Nine,
    /// The 'A' key
    A,
    /// The 'B' key
    B,
    /// The 'C' key
    C,
    /// The 'D' key
    D,
    /// The 'E' key
    E,
    /// The 'F' key
    F,
}

impl Chip8Key {
    /// Gets the key index in the chip8 keypad for the given key
    fn key_index(&self) -> usize {
        match self {
            Chip8Key::Zero => 0,
            Chip8Key::One => 1,
            Chip8Key::Two => 2,
            Chip8Key::Three => 3,
            Chip8Key::Four => 4,
            Chip8Key::Five => 5,
            Chip8Key::Six => 6,
            Chip8Key::Seven => 7,
            Chip8Key::Eight => 8,
            Chip8Key::Nine => 9,
            Chip8Key::A => 10,
            Chip8Key::B => 11,
            Chip8Key::C => 12,
            Chip8Key::D => 13,
            Chip8Key::E => 14,
            Chip8Key::F => 15,
        }
    }
}

/// Represents the state of a Chip8 key, either up or down
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyState {
    /// Represents the key being up, i.e. not pressed
    Up,
    /// Represents the key being down, i.e. pressed
    Down
}

/// Represents a 16-bit opcode
#[derive(Debug)]
struct OpCode {
    /// Full 16-bit opcode
    opcode: u16,
}

impl OpCode {
    /// Create a new opcode instance from a u16
    fn new(opcode: u16) -> Self {
        Self {
            opcode
        }
    }

    /// second-highest nibble of the opcode as u8
    fn x(&self) -> u8 {
        ((self.opcode & 0x0F00) >> 8) as u8
    }

    /// third-highest nibble of the opcode as u8
    fn y(&self) -> u8 {
        ((self.opcode & 0x00F0) >> 4) as u8
    }

    /// last (lowest) nibble of the opcode as u8
    fn n(&self) -> u8 {
        (self.opcode & 0x000F) as u8
    }

    /// lower byte (last two nibbles) of the opcode as u8
    fn nn(&self) -> u8 {
        (self.opcode & 0x00FF) as u8
    }

    /// second, third and fourth nibbles of the opcode as u16
    fn nnn(&self) -> u16 {
        self.opcode & 0x0FFF
    }
}

/// Represents a Chip8 interpreter
#[derive(Debug)]
pub struct Chip8<R: Rng> {
    /// [`MEMORY_SIZE`] bytes of memory
    ram: [u8; MEMORY_SIZE],
    /// frame buffer for drawing screen
    frame_buffer: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    /// A stack of [`STACK_SIZE`] 2-byte addresses
    stack: [u16; STACK_SIZE],
    /// A pointer to the current location in the stack
    stack_pointer: u8,
    /// A delay timer register
    delay_timer: u8,
    /// A sound timer register
    sound_timer: u8,
    /// The program counter
    program_counter: u16,
    /// An index register
    index_register: u16,
    /// [`VARIABLE_REGISTER_COUNT`] 8-bit variable registers
    variable_registers: [u8; VARIABLE_REGISTER_COUNT],
    /// The emulator type to interpret instructions as
    emulator_type: EmulatorType,
    /// A random number generator used to generate random numbers for certain instructions
    rng: R,
    /// The current state of the keypad
    keypad_state: [KeyState; NUM_KEYS],
    /// The state of the keypad on the previous loop through of the "put key into VX" instruction
    previous_keypad_state: [KeyState; NUM_KEYS],
}

impl<R: Rng> Chip8<R> {
    /// Creates a new Chip8 instance
    pub fn new(emulator_type: EmulatorType, rng: R) -> Self {
        let mut chip8 = Self {
            ram: [0; MEMORY_SIZE],
            frame_buffer: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: PROGRAM_START_ADDRESS,
            index_register: 0,
            variable_registers: [0; VARIABLE_REGISTER_COUNT],
            emulator_type,
            rng,
            keypad_state: [KeyState::Up; NUM_KEYS],
            previous_keypad_state: [KeyState::Up; NUM_KEYS],
        };

        chip8.ram[FONT_START_ADDRESS..FONT_START_ADDRESS + FONT.len()].copy_from_slice(&FONT);

        chip8
    }

    /// Gets the Chip-8 instances frame buffer
    pub fn frame_buffer(&self) -> &[u8; DISPLAY_WIDTH * DISPLAY_HEIGHT] {
        &self.frame_buffer
    }

    /// Returns whether the emulator should be playing a sound
    pub fn is_playing_sound(&self) -> bool {
        self.sound_timer > 0
    }

    /// Loads program into memory starting at address `{PROGRAM_START_ADDRESS}`
    pub fn load_program(&mut self, program: &[u8]) {
        let start_address = PROGRAM_START_ADDRESS as usize;
        self.ram[start_address..start_address + program.len()].copy_from_slice(program);
    }

    /// flags the key at key_index as down
    pub fn key_down(&mut self, key: Chip8Key) {
        self.keypad_state[key.key_index()] = KeyState::Down;
    }

    /// flags the key at key_index as up
    pub fn key_up(&mut self, key: Chip8Key) {
        self.keypad_state[key.key_index()] = KeyState::Up;
    }

    /// Decrements the delay and sound timers by 1
    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    /// gets the current KeyState for a key
    pub fn key_state(&self, key: Chip8Key) -> KeyState {
        self.keypad_state[key.key_index()]
    }

    /// Execute the instruction at the address pointed to by the program counter register
    /// and increment the program counter register so the next instruction can be executed on the
    /// next call.
    pub fn execute_next_instruction(&mut self) {
         let opcode = self.fetch_next_opcode();

        self.program_counter += 2;

        match opcode.opcode {
            0x00E0 => self.clear_screen(),
            0x00EE => self.return_from_subroutine(),
            0x1000..=0x1FFF => self.jump(opcode.nnn()),
            0x2000..=0x2FFF => self.call_subroutine(opcode.nnn()),
            0x3000..=0x3FFF => self.skip_instruction_if_vx_equals_nn(opcode.x(), opcode.nn()),
            0x4000..=0x4FFF => self.skip_instruction_if_vx_not_equals_nn(opcode.x(), opcode.nn()),
            0x5000..=0x5FFF => self.skip_instruction_if_vx_equals_vy(opcode.x(), opcode.y()),
            0x6000..=0x6FFF => self.set_variable_register(opcode.x(), opcode.nn()),
            0x7000..=0x7FFF => self.add_to_variable_register(opcode.x(), opcode.nn()),
            0x8000..=0x8FFF => {
                match opcode.n() {
                    0x0 => self.set_vx_to_vy(opcode.x(), opcode.y()),
                    0x1 => self.binary_or_vx_with_vy(opcode.x(), opcode.y()),
                    0x2 => self.binary_and_vx_with_vy(opcode.x(), opcode.y()),
                    0x3 => self.binary_xor_vx_with_vy(opcode.x(), opcode.y()),
                    0x4 => self.add_vy_to_vx(opcode.x(), opcode.y()),
                    0x5 => self.subtract_vy_from_vx(opcode.x(), opcode.y()),
                    0x6 => self.shift_vx_right(opcode.x(), opcode.y()),
                    0x7 => self.subtract_vx_from_vy_into_vx(opcode.x(), opcode.y()),
                    0xE => self.shift_vx_left(opcode.x(), opcode.y()),
                    _ => self.panic_for_invalid_opcode(opcode) // TODO test this case
                }
            },
            0x9000..=0x9FFF => self.skip_instruction_if_vx_not_equals_vy(opcode.x(), opcode.y()),
            0xA000..=0xAFFF => self.set_index_register(opcode.nnn()),
            0xB000..=0xBFFF => self.jump_with_offset(opcode.nnn()),
            0xC000..=0xCFFF => self.randomize_vx(opcode.x(), opcode.nn()),
            0xD000..=0xDFFF => self.draw(opcode.x(), opcode.y(), opcode.n()),
            0xE000..=0xEFFF => {
                match opcode.nn() {
                    0x9E => self.skip_if_key_down(opcode.x()),
                    0xA1 => self.skip_if_key_up(opcode.x()),
                    _ => self.panic_for_invalid_opcode(opcode) // TODO test this case
                }
            }
            0xF000..=0xFFFF => {
                match opcode.nn() {
                    0x07 => self.set_vx_to_delay_timer(opcode.x()),
                    0x0A => self.put_key_into_vx(opcode.x()),
                    0x15 => self.set_delay_timer_to_vx(opcode.x()),
                    0x18 => self.set_sound_timer_to_vx(opcode.x()),
                    0x1E => self.add_vx_to_index_register(opcode.x()),
                    0x29 => self.point_index_register_at_font_character(opcode.x()),
                    0x33 => self.put_vx_decimal_digits_into_memory(opcode.x()),
                    0x55 => self.store_variable_registers_to_memory(opcode.x()),
                    0x65 => self.load_variable_registers_from_memory(opcode.x()),
                    _ => self.panic_for_invalid_opcode(opcode) // TODO test this case
                }
            }
            _ => self.panic_for_invalid_opcode(opcode)
        }
    }

    /// Panics with a message indicating an invalid opcode was encountered.
    fn panic_for_invalid_opcode(&self, opcode: OpCode) {
        panic!("Encountered invalid opcode {:X}", opcode.opcode)
    }

    /// Fetches and returns the next opcode starting at the address pointed to by the program
    /// counter register.
    fn fetch_next_opcode(&mut self) -> OpCode {
        let high = self.ram[self.program_counter as usize];
        let low = self.ram[(self.program_counter + 1) as usize];

        let opcode = (high as u16) << 8 | low as u16;

        OpCode::new(opcode)
    }

    /// sets all values in the frame buffer to 0
    fn clear_screen(&mut self) {
        self.frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
    }

    /// sets the program counter register to nnn
    fn jump(&mut self, nnn: u16) {
        self.program_counter = nnn;
    }

    /// Sets variable register at index x to nn
    fn set_variable_register(&mut self, x: u8, nn: u8) {
        self.variable_registers[x as usize] = nn;
    }

    /// Adds nn to variable register at index x
    fn add_to_variable_register(&mut self, x: u8, nn: u8) {
        self.variable_registers[x as usize] = self.variable_registers[x as usize].wrapping_add(nn);
    }

    /// Sets the index register to nnn
    fn set_index_register(&mut self, nnn: u16) {
        self.index_register = nnn;
    }

    /// Draws an n pixel tall sprite from the memory location that the index register is holding to
    /// the frame buffer, at horizontal X coordinate held in variable register at index x and the Y
    /// coordinate held in the variable register at index y
    fn draw(&mut self, x: u8, y: u8, n: u8) {
        let sprite_memory_address = self.index_register as usize;
        let sprite_bytes = &self.ram[sprite_memory_address..sprite_memory_address + n as usize];
        let x_offset = self.variable_registers[x as usize] as usize % DISPLAY_WIDTH;
        let y_offset = self.variable_registers[y as usize] as usize % DISPLAY_HEIGHT;

        // Reset the VF register. We will flip it if any pixes go from ON to OFF
        self.variable_registers[0xF] = 0;

        // iterate over the sprite bytes
        for (row, byte) in sprite_bytes.iter().enumerate() {
            // iterate over the bits in the current sprite byte
            for bit_index in 0..8 {
                let pixel_x_index = x_offset + bit_index;
                let pixel_y_index = y_offset + row;

                // Don't draw sprite pixels if they go off the edge of the screen
                if pixel_x_index < DISPLAY_WIDTH && pixel_y_index < DISPLAY_HEIGHT {
                    let sprite_pixel_value= (byte >> (7 - bit_index)) & 0x01;
                    let frame_buffer_pixel_index = pixel_y_index * DISPLAY_WIDTH + pixel_x_index;
                    let frame_buffer_pixel_value = self.frame_buffer[frame_buffer_pixel_index];

                    // If a pixel was on but is now off flip the VF register
                    if frame_buffer_pixel_value == 1 && sprite_pixel_value == 1 {
                        self.variable_registers[0xF] = 1;
                    }

                    self.frame_buffer[frame_buffer_pixel_index] = frame_buffer_pixel_value ^ sprite_pixel_value;
                }
            }
        }
    }

    /// Pushes the current program counter address to the stack and jumps to a new address
    fn call_subroutine(&mut self, nnn: u16) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = nnn;
    }

    /// Pops an address from the stack and sets the program counter to it
    fn return_from_subroutine(&mut self) {
        self.stack_pointer -= 1;
        let address = self.stack[self.stack_pointer as usize];
        self.program_counter = address;
    }

    /// Skips an instruction by incrementing program counter by 2 if variable register at index x == nn
    fn skip_instruction_if_vx_equals_nn(&mut self, x: u8, nn: u8) {
        if self.variable_registers[x as usize] == nn {
            self.program_counter += 2;
        }
    }

    /// Skips an instruction by incrementing program counter by 2 if variable register at index x != nn
    fn skip_instruction_if_vx_not_equals_nn(&mut self, x: u8, nn: u8) {
        if self.variable_registers[x as usize] != nn {
            self.program_counter += 2;
        }
    }

    /// Skips an instruction by incrementing program counter by 2 if value in variable register at index x == value in variable register at index y
    fn skip_instruction_if_vx_equals_vy(&mut self, x: u8, y: u8) {
        if self.variable_registers[x as usize] == self.variable_registers[y as usize] {
            self.program_counter += 2;
        }
    }

    /// Skips an instruction by incrementing program counter by 2 if value in variable register at index x != value in variable register at index y
    fn skip_instruction_if_vx_not_equals_vy(&mut self, x: u8, y: u8) {
        if self.variable_registers[x as usize] != self.variable_registers[y as usize] {
            self.program_counter += 2;
        }
    }

    /// Sets value in variable register at index x to the value in variable register at index y
    fn set_vx_to_vy(&mut self, x: u8, y: u8) {
        self.variable_registers[x as usize] = self.variable_registers[y as usize];
    }

    /// Sets value in variable register x to the result of doing a binary or with the value
    /// in variable register x and the value in variable register y
    fn binary_or_vx_with_vy(&mut self, x: u8, y: u8) {
        self.variable_registers[x as usize] |= self.variable_registers[y as usize];
    }

    /// Sets value in variable register x to the result of doing a binary and with the value
    /// in variable register x and the value in variable register y
    fn binary_and_vx_with_vy(&mut self, x: u8, y: u8) {
        self.variable_registers[x as usize] &= self.variable_registers[y as usize];
    }

    /// Sets value in variable register x to the result of doing a binary xor with the value
    /// in variable register x and the value in variable register y
    fn binary_xor_vx_with_vy(&mut self, x: u8, y: u8) {
        self.variable_registers[x as usize] ^= self.variable_registers[y as usize];
    }

    /// Adds VY to VX. If the sum of VY and VX would overflow the 8-bit register VF is set to 1,
    /// otherwise it's set to 0
    fn add_vy_to_vx(&mut self, x: u8, y: u8) {
        let x_val = self.variable_registers[x as usize];
        let y_val = self.variable_registers[y as usize];

        if x_val > 0xFF - y_val {
            self.variable_registers[0xF] = 1;
        } else {
            self.variable_registers[0xF] = 0;
        }

        self.variable_registers[x as usize] = x_val.wrapping_add(y_val);
    }

    /// Subtracts VY from VX. If VX is less than VY and the subtraction would overflow VF is set to
    /// 0, otherwise it's set to 1
    fn subtract_vy_from_vx(&mut self, x: u8, y: u8) {
        let x_val = self.variable_registers[x as usize];
        let y_val = self.variable_registers[y as usize];

        if x_val < y_val {
            self.variable_registers[0xF] = 0;
        } else {
            self.variable_registers[0xF] = 1;
        }

        self.variable_registers[x as usize] = x_val.wrapping_sub(y_val);
    }

    /// Subtracts VX from VY and puts the result in VX. If VY is less than VX and the subtraction
    /// would overflow VF is set to 0, otherwise it's set to 1
    fn subtract_vx_from_vy_into_vx(&mut self, x: u8, y: u8) {
        let x_val = self.variable_registers[x as usize];
        let y_val = self.variable_registers[y as usize];

        if y_val < x_val {
            self.variable_registers[0xF] = 0;
        }
        else {
            self.variable_registers[0xF] = 1;
        }

        self.variable_registers[x as usize] = y_val.wrapping_sub(x_val);
    }

    /// If emulator type is CosmacVip sets VX to VY and shifts VX right 1 bit.
    /// If emulator type is Chip48 just shifts VX right 1 bit.
    /// In both cases sets VF to the shifted out bit
    fn shift_vx_right(&mut self, x: u8, y: u8) {
        if self.emulator_type == EmulatorType::CosmacVip {
            self.variable_registers[x as usize] = self.variable_registers[y as usize];
        }

        self.variable_registers[0xF] = self.variable_registers[x as usize] & 0x01;
        self.variable_registers[x as usize] >>= 1;
    }

    /// If emulator type is CosmacVip sets VX to VY and shifts VX left 1 bit.
    /// If emulator type is Chip48 just shifts VX left 1 bit.
    /// In both cases sets VF to the shifted out bit
    fn shift_vx_left(&mut self, x: u8, y: u8) {
        if self.emulator_type == EmulatorType::CosmacVip {
            self.variable_registers[x as usize] = self.variable_registers[y as usize];
        }

        self.variable_registers[0xF] = (self.variable_registers[x as usize] >> 7) & 0x01;
        self.variable_registers[x as usize] <<= 1;
    }

    /// If the emulator type is CosmacVip this will jump to address NNN plus the value in V0
    /// If the emulator type is Chip48 this will jump to address XNN plus the value in VX
    fn jump_with_offset(&mut self, nnn: u16) {
        let offset = match self.emulator_type {
            EmulatorType::CosmacVip => self.variable_registers[0] as u16,
            EmulatorType::Chip48 => self.variable_registers[(nnn & 0x0F00) as usize >> 8] as u16,
        };

        self.jump(nnn.wrapping_add(offset));
    }

    /// Generates a random number, does a binary and with nn and puts the value in Vx
    fn randomize_vx(&mut self, x: u8, nn: u8) {
        let random_number = self.rng.random::<u8>();
        self.variable_registers[x as usize] = random_number & nn;
    }

    /// Skips one instruction if the key at index held in Vx is down
    fn skip_if_key_down(&mut self, x: u8) {
        let key_index = self.variable_registers[x as usize] as usize;
        if self.keypad_state[key_index] == KeyState::Down {
            self.program_counter += 2;
        }
    }

    /// Skips one instruction if the key at index held in Vx is up
    fn skip_if_key_up(&mut self, x: u8) {
        let key_index = self.variable_registers[x as usize] as usize;
        if self.keypad_state[key_index] == KeyState::Up {
            self.program_counter += 2;
        }
    }

    /// Sets Vx to the value in the delay timer
    fn set_vx_to_delay_timer(&mut self, x: u8) {
       self.variable_registers[x as usize] = self.delay_timer;
    }

    /// Sets the delay timer to the current value in Vx
    fn set_delay_timer_to_vx(&mut self, x: u8) {
        self.delay_timer = self.variable_registers[x as usize];
    }

    /// Sets the sound timer to the current value in Vx
    fn set_sound_timer_to_vx(&mut self, x: u8) {
        self.sound_timer = self.variable_registers[x as usize];
    }

    /// Adds Vx to the index register(x)
    fn add_vx_to_index_register(&mut self, x: u8) {
        self.index_register = self.index_register.wrapping_add(self.variable_registers[x as usize] as u16);
    }

    /// If a key is pressed this puts the key into Vx, otherwise it decrements the program counter by 2
    fn put_key_into_vx(&mut self, x: u8) {
        let key_option = self.keypad_state.iter()
            .enumerate()
            .position(|(i, key_state)| *key_state == KeyState::Up && self.previous_keypad_state[i] == KeyState::Down);

        if let Some(key_index) = key_option {
            self.previous_keypad_state = self.keypad_state;
            self.variable_registers[x as usize] = key_index as u8;
        } else {
            self.previous_keypad_state = self.keypad_state;
            self.program_counter -= 2;
        }
    }

    /// Sets the value of the index register to the address of font character held in the last nibble
    /// of Vx
    fn point_index_register_at_font_character(&mut self, x: u8) {
        let vx = self.variable_registers[x as usize];
        let character_index = vx & 0x0F;
        let character_address = FONT_START_ADDRESS as u16 + (character_index as u16 * 5);

        self.index_register = character_address;
    }

    /// Instruction to put the decimal digits of the value stored in Vx into memory starting at
    /// the address held in the index register
    fn put_vx_decimal_digits_into_memory(&mut self, x: u8) {
        let vx = self.variable_registers[x as usize];
        let digits: Vec<u8> = (0..3).rev().map(|i| (vx / (10u8.pow(i))) % 10u8).collect();
        let start_address = self.index_register as usize;

        for (offset, digit) in digits.iter().enumerate() {
            self.ram[start_address + offset] = *digit;
        }
    }

    /// Ambiguous instruction. This instruction stores the values in the variable registers V0 to Vx
    /// inclusively to memory beginning at the address held in the index register.
    /// For the COSMAC-VIP the index register will be incremented after each register is stored.
    /// For the CHIP-48 the index register will not be updated
    fn store_variable_registers_to_memory(&mut self, x: u8) {
        let start_address = self.index_register as usize;

        for i in 0..=x as usize {
            self.ram[start_address + i] = self.variable_registers[i];

            if self.emulator_type == EmulatorType::CosmacVip {
                self.index_register += 1;
            }
        }
    }

    /// Ambiguous instruction. This instruction loads values from memory starting at the address
    /// held in the index register into the variable registers V0 to Vx inclusively.
    /// For the COSMAC-VIP the index register will be incremented after each register is loaded.
    /// For the CHIP-48 the index register will not be updated
    fn load_variable_registers_from_memory(&mut self, x: u8) {
        let start_address = self.index_register as usize;

        for i in 0..=x as usize {
            self.variable_registers[i] = self.ram[start_address + i];

            if self.emulator_type == EmulatorType::CosmacVip {
                self.index_register += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::{StdRng};
    use rand::SeedableRng;
    use super::*;

    fn draw_test_sprite(test_frame_buffer: &mut [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
                        x_offset: u8,
                        y_offset: u8,
                        sprite_bytes: &[u8]) {
        for (row, byte) in sprite_bytes.iter().enumerate() {
            for bit in 0..8 {
                let sprite_bit_value = (byte >> (7 - bit)) & 0x01;
                let x_index = (x_offset + bit) as usize;
                let y_index = y_offset as usize + row;

                if x_index < DISPLAY_WIDTH && y_index < DISPLAY_HEIGHT {
                    test_frame_buffer[y_index * DISPLAY_WIDTH + x_index] ^= sprite_bit_value;
                }
            }
        }
    }

    #[test]
    fn can_create_new_chip_8() {
        let chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        let mut expected_ram = [0; MEMORY_SIZE];
        expected_ram[0x050..0x050 + FONT.len()].copy_from_slice(&FONT);

        let expected_frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        let expected_stack = [0; STACK_SIZE];
        let expected_stack_pointer = 0;
        let expected_delay_timer = 0;
        let expected_sound_timer = 0;
        let expected_program_counter = PROGRAM_START_ADDRESS;
        let expected_index_register = 0;
        let expected_variable_registers = [0; VARIABLE_REGISTER_COUNT];
        let expected_keypad = [KeyState::Up; NUM_KEYS];

        assert_eq!(expected_ram, chip8.ram);
        assert_eq!(expected_frame_buffer, chip8.frame_buffer);
        assert_eq!(expected_stack, chip8.stack);
        assert_eq!(expected_stack_pointer, chip8.stack_pointer);
        assert_eq!(expected_delay_timer, chip8.delay_timer);
        assert_eq!(expected_sound_timer, chip8.sound_timer);
        assert_eq!(expected_program_counter, chip8.program_counter);
        assert_eq!(expected_index_register, chip8.index_register);
        assert_eq!(expected_variable_registers, chip8.variable_registers);
        assert_eq!(EmulatorType::CosmacVip, chip8.emulator_type);
        assert_eq!(expected_keypad, chip8.keypad_state);
    }

    #[test]
    fn can_create_new_chip_8_with_chip_48_type() {
        let chip8 = Chip8::new(EmulatorType::Chip48, rand::rng());

        assert_eq!(EmulatorType::Chip48, chip8.emulator_type);
    }

    #[test]
    fn can_create_opcode() {
        let opcode = OpCode::new(0x1234);

        assert_eq!(0x1234, opcode.opcode);
        assert_eq!(2, opcode.x());
        assert_eq!(3, opcode.y());
        assert_eq!(4, opcode.n());
        assert_eq!(0x34, opcode.nn());
        assert_eq!(0x234, opcode.nnn());
    }

    #[test]
    fn can_load_program() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        let program = [0x00, 0xE0, 0x12, 0x34, 0x56, 0x78];
        chip8.load_program(&program);

        let start_address = PROGRAM_START_ADDRESS as usize;
        assert_eq!(program, chip8.ram[start_address..start_address + program.len()]);
    }

    #[test]
    fn can_handle_key_down() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        assert_eq!(KeyState::Up, chip8.keypad_state[Chip8Key::C.key_index()]);

        chip8.key_down(Chip8Key::C);

        assert_eq!(KeyState::Down, chip8.keypad_state[Chip8Key::C.key_index()]);
    }

    #[test]
    fn can_handle_key_up() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.key_down(Chip8Key::C);

        assert_eq!(KeyState::Down, chip8.keypad_state[Chip8Key::C.key_index()]);

        chip8.key_up(Chip8Key::C);

        assert_eq!(KeyState::Up, chip8.keypad_state[Chip8Key::C.key_index()]);
    }

    #[test]
    fn can_get_key_state() {
        let key = Chip8Key::C;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.key_down(key);
        assert_eq!(KeyState::Down, chip8.key_state(key));

        chip8.key_up(key);
        assert_eq!(KeyState::Up, chip8.key_state(key));
    }

    #[test]
    fn can_decrement_timers() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.delay_timer = 100;
        chip8.sound_timer = 100;

        chip8.decrement_timers();

        assert_eq!(99, chip8.delay_timer);
        assert_eq!(99, chip8.sound_timer);
    }

    #[test]
    fn decrement_timers_does_nothing_if_timers_are_at_zero() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.delay_timer = 0;
        chip8.sound_timer = 0;

        chip8.decrement_timers();

        assert_eq!(0, chip8.delay_timer);
        assert_eq!(0, chip8.sound_timer);
    }

    #[test]
    fn can_fetch_next_opcode() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x00;
        chip8.ram[0x201] = 0xE0;
        chip8.program_counter = 0x200;

        let opcode = chip8.fetch_next_opcode();

        assert_eq!(0x00E0, opcode.opcode);
    }

    #[test]
    fn can_clear_screen() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.frame_buffer = [1; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        chip8.clear_screen();
        assert_eq!([0; DISPLAY_WIDTH * DISPLAY_HEIGHT], chip8.frame_buffer);
    }

    #[test]
    fn can_jump() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x200;
        chip8.jump(0x300);
        assert_eq!(0x300, chip8.program_counter);
    }

    #[test]
    fn can_set_variable_register() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.set_variable_register(0x2, 0x34);
        chip8.set_variable_register(0x7, 0xAA);

        assert_eq!(0x34, chip8.variable_registers[2]);
        assert_eq!(0xAA, chip8.variable_registers[7]);
    }

    #[test]
    fn can_add_to_variable_register() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.set_variable_register(0x2, 0x34);
        chip8.set_variable_register(0x7, 0xAA);

        assert_eq!(0x34, chip8.variable_registers[2]);
        assert_eq!(0xAA, chip8.variable_registers[7]);

        chip8.add_to_variable_register(0x2, 0x34);
        chip8.add_to_variable_register(0x7, 0x12);

        assert_eq!(0x68, chip8.variable_registers[2]);
        assert_eq!(0xBC, chip8.variable_registers[7]);
    }

    #[test]
    fn add_to_variable_register_handles_overflow() {
        let initial_value: u8 = 0xF3;
        let value_to_add: u8 = 0x34;
        let expected_result = initial_value.wrapping_add(value_to_add);
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.set_variable_register(0x2, initial_value);

        assert_eq!(initial_value, chip8.variable_registers[2]);

        chip8.add_to_variable_register(0x2, value_to_add);

        assert_eq!(expected_result, chip8.variable_registers[2]);
    }

    #[test]
    fn can_set_index_register() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.set_index_register(0x300);
        assert_eq!(0x300, chip8.index_register);
    }

    #[test]
    fn can_draw_sprite() {
        let x_offset = 34;
        let y_offset = 12;
        let sprite_bytes = [0b11111111, 0b01010101, 0b00000000, 0b11011101];
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        let mut test_frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

        draw_test_sprite(&mut test_frame_buffer, x_offset, y_offset, &sprite_bytes);

        chip8.frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        chip8.ram[0x300..0x304].copy_from_slice(&sprite_bytes);
        chip8.set_index_register(0x300);
        chip8.set_variable_register(0x2, x_offset);
        chip8.set_variable_register(0x3, y_offset);

        chip8.draw(0x2, 0x3, 0x4);

        assert_eq!(test_frame_buffer, chip8.frame_buffer);
    }

    #[test]
    fn draw_flips_pixel_state_correctly() {
        let x_offset = 34;
        let y_offset = 12;
        let sprite_bytes = [0b11111111, 0b01010101];
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        let mut test_frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

        test_frame_buffer[12 * DISPLAY_WIDTH + 34..12 * DISPLAY_WIDTH + 34 + 8]
            .copy_from_slice(&[1, 1, 1, 1, 0, 0, 0, 0]);

        test_frame_buffer[13 * DISPLAY_WIDTH + 34..13 * DISPLAY_WIDTH + 34 + 8]
            .copy_from_slice(&[1, 1, 1, 1, 0, 0, 0, 0]);
        
        draw_test_sprite(&mut test_frame_buffer, x_offset, y_offset, &sprite_bytes);

        chip8.frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

        chip8.frame_buffer[12 * DISPLAY_WIDTH + 34..12 * DISPLAY_WIDTH + 34 + 8]
            .copy_from_slice(&[1, 1, 1, 1, 0, 0, 0, 0]);

        chip8.frame_buffer[13 * DISPLAY_WIDTH + 34..13 * DISPLAY_WIDTH + 34 + 8]
            .copy_from_slice(&[1, 1, 1, 1, 0, 0, 0, 0]);

        chip8.ram[0x300..0x302].copy_from_slice(&sprite_bytes);
        chip8.set_index_register(0x300);
        chip8.set_variable_register(0x2, x_offset);
        chip8.set_variable_register(0x3, y_offset);

        chip8.draw(0x2, 0x3, 0x2);

        assert_eq!(test_frame_buffer, chip8.frame_buffer);
    }

    #[test]
    fn drawing_sprites_near_edge_does_not_wrap() {
        let x_offset = 60;
        let y_offset = 30;
        let sprite_bytes = [0b11111111, 0b01010101, 0b00000000, 0b11011101];
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        let mut test_frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

        draw_test_sprite(&mut test_frame_buffer, x_offset, y_offset, &sprite_bytes);

        chip8.frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        chip8.ram[0x300..0x304].copy_from_slice(&sprite_bytes);
        chip8.set_index_register(0x300);
        chip8.set_variable_register(0x2, x_offset);
        chip8.set_variable_register(0x3, y_offset);

        chip8.draw(0x2, 0x3, 0x4);

        assert_eq!(test_frame_buffer, chip8.frame_buffer);
    }

    #[test]
    fn can_call_subroutine() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x202;

        chip8.call_subroutine(0x300);

        assert_eq!(0x202, chip8.stack[0]);
        assert_eq!(1, chip8.stack_pointer);
        assert_eq!(0x300, chip8.program_counter);
    }

    #[test]
    fn can_return_from_subroutine() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.stack[0] = 0x200;

        chip8.call_subroutine(0x300);
        assert_eq!(0x200, chip8.stack[0]);
        assert_eq!(1, chip8.stack_pointer);
        assert_eq!(0x300, chip8.program_counter);

        chip8.return_from_subroutine();
        assert_eq!(0x200, chip8.program_counter);
        assert_eq!(0, chip8.stack_pointer);
    }

    #[test]
    fn skip_instruction_if_vx_equals_nn_skips_when_equal() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_equals_nn(0x2, 0x34);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_equals_nn_does_not_skips_when_not_equal() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_equals_nn(0x2, 0x35);

        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_not_equals_nn_skips_when_not_equal() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_not_equals_nn(0x2, 0x35);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_not_equals_nn_does_not_skips_when_equal() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_not_equals_nn(0x2, 0x34);

        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_equals_vy_skips_when_equal() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0x3] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_equals_vy(0x2, 0x3);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_equals_vy_does_not_skips_when_not_equal() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0x3] = 0x35;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_equals_vy(0x2, 0x3);

        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_not_equals_vy_skips_when_not_equal() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0x3] = 0x35;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_not_equals_vy(0x2, 0x3);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_not_equals_vy_does_not_skips_when_equal() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0x3] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_not_equals_vy(0x2, 0x3);

        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn can_set_vx_to_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xF] = 0xAF;
        chip8.program_counter = 0x202;

        chip8.set_vx_to_vy(0x2, 0xF);

        assert_eq!(0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn can_binary_or_vx_with_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xC] = 0xAF;

        chip8.binary_or_vx_with_vy(0x2, 0xC);

        assert_eq!(0x34 | 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn can_binary_and_vx_with_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xC] = 0xAF;

        chip8.binary_and_vx_with_vy(0x2, 0xC);

        assert_eq!(0x34 & 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn can_binary_xor_vx_with_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xC] = 0xAF;

        chip8.binary_xor_vx_with_vy(0x2, 0xC);

        assert_eq!(0x34 ^ 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn can_add_vy_to_vx() {
        let x_val: u8 = 0x34;
        let y_val: u8 = 0xAF;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.variable_registers[0xF] = 0x01;

        chip8.add_vy_to_vx(0x2, 0xC);

        assert_eq!(x_val + y_val, chip8.variable_registers[0x2]);
        assert_eq!(0x0, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_add_vy_to_vx_with_carry() {
        let x_val: u8 = 0xD4;
        let y_val: u8 = 0xAF;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.variable_registers[0xF] = 0x00;

        chip8.add_vy_to_vx(0x2, 0xC);

        assert_eq!(x_val.wrapping_add(y_val), chip8.variable_registers[0x2]);
        assert_eq!(0x1, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_subtract_vy_from_vx() {
        let x_val: u8 = 0xAF;
        let y_val: u8 = 0x34;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.variable_registers[0xF] = 0x00;

        chip8.subtract_vy_from_vx(0x2, 0xC);

        assert_eq!(x_val.wrapping_sub(y_val), chip8.variable_registers[0x2]);
        assert_eq!(0x1, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_subtract_vy_from_vx_with_borrow() {
        let x_val: u8 = 0x34;
        let y_val: u8 = 0xAF;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.variable_registers[0xF] = 0x01;

        chip8.subtract_vy_from_vx(0x2, 0xC);

        assert_eq!(x_val.wrapping_sub(y_val), chip8.variable_registers[0x2]);
        assert_eq!(0x0, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_subtract_vx_from_vy_into_vx() {
        let x_val: u8 = 0x34;
        let y_val: u8 = 0xAF;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.variable_registers[0xF] = 0x00;

        chip8.subtract_vx_from_vy_into_vx(0x2, 0xC);

        assert_eq!(y_val.wrapping_sub(x_val), chip8.variable_registers[0x2]);
        assert_eq!(0x1, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_subtract_vx_from_vy_into_vx_with_borrow() {
        let x_val: u8 = 0xAF;
        let y_val: u8 = 0x34;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.variable_registers[0xF] = 0x01;

        chip8.subtract_vx_from_vy_into_vx(0x2, 0xC);

        assert_eq!(y_val.wrapping_sub(x_val), chip8.variable_registers[0x2]);
        assert_eq!(0x0, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_shift_vx_right_for_cosmac_vip() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x00;
        chip8.variable_registers[0xC] = 0x13;

        chip8.shift_vx_right(0x2, 0xC);

        assert_eq!(0x13 >> 1, chip8.variable_registers[0x2]);
        assert_eq!(0x1, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_shift_vx_right_for_chip_48() {
        let mut chip8 = Chip8::new(EmulatorType::Chip48, rand::rng());
        chip8.variable_registers[0x2] = 0x13;
        chip8.variable_registers[0xC] = 0x34;

        chip8.shift_vx_right(0x2, 0xC);

        assert_eq!(0x13 >> 1, chip8.variable_registers[0x2]);
        assert_eq!(0x1, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_shift_vx_left_for_cosmac_vip() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x00;
        chip8.variable_registers[0xC] = 0x13;
        chip8.variable_registers[0xF] = 0x01;

        chip8.shift_vx_left(0x2, 0xC);

        assert_eq!(0x13 << 1, chip8.variable_registers[0x2]);
        assert_eq!(0x0, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_shift_vx_left_for_chip_48() {
        let mut chip8 = Chip8::new(EmulatorType::Chip48, rand::rng());
        chip8.variable_registers[0x2] = 0x13;
        chip8.variable_registers[0xC] = 0x34;
        chip8.variable_registers[0xF] = 0x01;

        chip8.shift_vx_left(0x2, 0xC);

        assert_eq!(0x13 << 1, chip8.variable_registers[0x2]);
        assert_eq!(0x0, chip8.variable_registers[0xF]);
    }

    #[test]
    fn can_jump_with_offset_for_cosmac_vip() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x202;
        chip8.variable_registers[0x0] = 0x12;

        chip8.jump_with_offset(0x123);

        assert_eq!(0x12 + 0x123, chip8.program_counter);
    }

    #[test]
    fn jump_with_offset_does_wrapping_add_for_cosmac_vip() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x202;
        chip8.variable_registers[0x0] = 0x002;

        chip8.jump_with_offset(0xFFF);

        assert_eq!(0x002u16.wrapping_add(0xFFF), chip8.program_counter);
    }

    #[test]
    fn can_jump_with_offset_for_chip_48() {
        let mut chip8 = Chip8::new(EmulatorType::Chip48, rand::rng());
        chip8.program_counter = 0x202;
        chip8.variable_registers[0x2] = 0x12;

        chip8.jump_with_offset(0x223);

        assert_eq!(0x12 + 0x223, chip8.program_counter);
    }

    #[test]
    fn jump_with_offset_does_wrapping_add_for_chip_48() {
        let mut chip8 = Chip8::new(EmulatorType::Chip48, rand::rng());
        chip8.program_counter = 0x202;
        chip8.variable_registers[0xF] = 0x02;

        chip8.jump_with_offset(0xFFF);

        assert_eq!(0x02u16.wrapping_add(0xFFF), chip8.program_counter);
    }
    
    #[test]
    fn can_randomize_vx() {
        let rng = StdRng::seed_from_u64(0);
        let mut test_rng = rng.clone();
        
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rng);
        
        chip8.randomize_vx(2, 0x34);
        
        let random_val = test_rng.random::<u8>();
        assert_eq!(random_val & 0x34, chip8.variable_registers[2])
    }

    #[test]
    fn can_skip_if_key_down() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x202;
        chip8.variable_registers[0x4] = Chip8Key::C.key_index() as u8;

        chip8.key_down(Chip8Key::C);
        chip8.skip_if_key_down(0x4);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn can_skip_if_key_up() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x202;
        chip8.variable_registers[0x4] = Chip8Key::C.key_index() as u8;

        chip8.key_up(Chip8Key::C);
        chip8.skip_if_key_up(0x4);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn can_set_vx_to_delay_timer() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.delay_timer = 0x34;

        chip8.set_vx_to_delay_timer(0x3);

        assert_eq!(0x34, chip8.variable_registers[0x3]);
    }

    #[test]
    fn can_set_delay_timer_to_vx() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 0x34;

        chip8.set_delay_timer_to_vx(0x3);

        assert_eq!(0x34, chip8.delay_timer);
    }

    #[test]
    fn can_set_sound_timer_to_vx() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 0x34;

        chip8.set_sound_timer_to_vx(0x3);

        assert_eq!(0x34, chip8.sound_timer);
    }

    #[test]
    fn can_add_vx_to_index_register() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 0x34;
        chip8.index_register = 0x22;

        chip8.add_vx_to_index_register(0x3);

        assert_eq!(0x34 + 0x22, chip8.index_register);
    }

    #[test]
    fn can_add_vx_to_index_register_with_overflow() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 0xFF;
        chip8.index_register = 0x22;

        chip8.add_vx_to_index_register(0x3);

        assert_eq!(0xFFu16.wrapping_add(0x22), chip8.index_register);
    }

    #[test]
    fn put_key_into_vx_puts_correct_key_into_vx() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.previous_keypad_state[Chip8Key::Four.key_index()] = KeyState::Down;
        chip8.previous_keypad_state[Chip8Key::C.key_index()] = KeyState::Down;

        chip8.put_key_into_vx(0x3);

        chip8.keypad_state[Chip8Key::C.key_index()] = KeyState::Up;
        chip8.keypad_state[Chip8Key::Four.key_index()] = KeyState::Up;

        chip8.put_key_into_vx(0x3);

        assert_eq!(Chip8Key::Four.key_index() as u8, chip8.variable_registers[0x3]);
    }

    #[test]
    fn put_key_into_vx_decrements_program_counter_if_no_key_has_been_pressed() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x202;

        chip8.put_key_into_vx(0x3);

        assert_eq!(0x200, chip8.program_counter);
    }

    #[test]
    fn can_point_index_register_at_font_character() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 0x0C;

        chip8.point_index_register_at_font_character(0x3);

        assert_eq!(FONT_START_ADDRESS as u16 + (0xC * 5), chip8.index_register);
    }

    #[test]
    fn can_put_vx_decimal_digits_into_memory_for_zero() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 0;
        chip8.index_register = 0x234;
        chip8.ram[0x234] = 1;
        chip8.ram[0x235] = 2;
        chip8.ram[0x236] = 3;

        chip8.put_vx_decimal_digits_into_memory(0x3);

        assert_eq!(0, chip8.ram[0x234]);
        assert_eq!(0, chip8.ram[0x235]);
        assert_eq!(0, chip8.ram[0x236]);
    }

    #[test]
    fn can_put_vx_decimal_digits_into_memory_for_one_digit_number() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 3;
        chip8.index_register = 0x234;

        chip8.put_vx_decimal_digits_into_memory(0x3);

        assert_eq!(0, chip8.ram[0x234]);
        assert_eq!(0, chip8.ram[0x235]);
        assert_eq!(3, chip8.ram[0x236]);
    }

    #[test]
    fn can_put_vx_decimal_digits_into_memory_for_two_digit_number() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 43;
        chip8.index_register = 0x234;

        chip8.put_vx_decimal_digits_into_memory(0x3);

        assert_eq!(0, chip8.ram[0x234]);
        assert_eq!(4, chip8.ram[0x235]);
        assert_eq!(3, chip8.ram[0x236]);
    }

    #[test]
    fn can_put_vx_decimal_digits_into_memory_for_three_digit_number() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x3] = 243;
        chip8.index_register = 0x234;

        chip8.put_vx_decimal_digits_into_memory(0x3);

        assert_eq!(2, chip8.ram[0x234]);
        assert_eq!(4, chip8.ram[0x235]);
        assert_eq!(3, chip8.ram[0x236]);
    }

    #[test]
    fn can_store_variable_registers_to_memory_for_cosmac_vip() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x0] = 0x12;
        chip8.variable_registers[0x1] = 0x23;
        chip8.variable_registers[0x2] = 0x45;
        chip8.variable_registers[0x3] = 0x67;
        chip8.index_register = 0xC00;

        chip8.store_variable_registers_to_memory(0x3);

        assert_eq!(0x12, chip8.ram[0xC00]);
        assert_eq!(0x23, chip8.ram[0xC01]);
        assert_eq!(0x45, chip8.ram[0xC02]);
        assert_eq!(0x67, chip8.ram[0xC03]);
        assert_eq!(0xC04, chip8.index_register);
    }

    #[test]
    fn can_store_variable_registers_to_memory_for_chip_48() {
        let mut chip8 = Chip8::new(EmulatorType::Chip48, rand::rng());

        chip8.variable_registers[0x0] = 0x12;
        chip8.variable_registers[0x1] = 0x23;
        chip8.variable_registers[0x2] = 0x45;
        chip8.variable_registers[0x3] = 0x67;
        chip8.index_register = 0xC00;

        chip8.store_variable_registers_to_memory(0x3);

        assert_eq!(0x12, chip8.ram[0xC00]);
        assert_eq!(0x23, chip8.ram[0xC01]);
        assert_eq!(0x45, chip8.ram[0xC02]);
        assert_eq!(0x67, chip8.ram[0xC03]);
        assert_eq!(0xC00, chip8.index_register);
    }

    #[test]
    fn can_load_variable_registers_from_memory_for_cosmac_vip() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.ram[0xC00] = 0x12;
        chip8.ram[0xC01] = 0x23;
        chip8.ram[0xC02] = 0x45;
        chip8.ram[0xC03] = 0x67;
        chip8.index_register = 0xC00;

        chip8.load_variable_registers_from_memory(0x3);

        assert_eq!(0x12, chip8.variable_registers[0x0]);
        assert_eq!(0x23, chip8.variable_registers[0x1]);
        assert_eq!(0x45, chip8.variable_registers[0x2]);
        assert_eq!(0x67, chip8.variable_registers[0x3]);
        assert_eq!(0xC04, chip8.index_register);
    }

    #[test]
    fn can_load_variable_registers_from_memory_for_chip_48() {
        let mut chip8 = Chip8::new(EmulatorType::Chip48, rand::rng());

        chip8.ram[0xC00] = 0x12;
        chip8.ram[0xC01] = 0x23;
        chip8.ram[0xC02] = 0x45;
        chip8.ram[0xC03] = 0x67;
        chip8.index_register = 0xC00;

        chip8.load_variable_registers_from_memory(0x3);

        assert_eq!(0x12, chip8.variable_registers[0x0]);
        assert_eq!(0x23, chip8.variable_registers[0x1]);
        assert_eq!(0x45, chip8.variable_registers[0x2]);
        assert_eq!(0x67, chip8.variable_registers[0x3]);
        assert_eq!(0xC00, chip8.index_register);
    }

    #[test]
    fn execute_next_instruction_can_execute_clear_screen() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x00;
        chip8.ram[0x201] = 0xE0;
        chip8.program_counter = 0x200;
        chip8.frame_buffer = [1; DISPLAY_WIDTH * DISPLAY_HEIGHT];

        chip8.execute_next_instruction();

        assert_eq!([0; DISPLAY_WIDTH * DISPLAY_HEIGHT], chip8.frame_buffer);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn execute_next_instruction_can_execute_jump() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x12;
        chip8.ram[0x201] = 0x34;
        chip8.program_counter = 0x200;

        chip8.execute_next_instruction();

        assert_eq!(0x234, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_set_variable_register() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x63;
        chip8.ram[0x201] = 0xBC;
        chip8.program_counter = 0x200;

        chip8.execute_next_instruction();

        assert_eq!(0xBC, chip8.variable_registers[0x03]);
        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_add_to_variable_register() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x73;
        chip8.ram[0x201] = 0xBC;
        chip8.program_counter = 0x200;
        chip8.variable_registers[0x03] = 0x12;

        chip8.execute_next_instruction();

        assert_eq!(0xCE, chip8.variable_registers[0x03]);
        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_set_index_register() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0xA3;
        chip8.ram[0x201] = 0xBC;
        chip8.program_counter = 0x200;

        chip8.execute_next_instruction();

        assert_eq!(0x3BC, chip8.index_register);
        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_draw() {
        let x_offset = 34;
        let y_offset = 12;
        let sprite_bytes = [0b11111111, 0b01010101, 0b00000000, 0b11011101];
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        let mut test_frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

        draw_test_sprite(&mut test_frame_buffer, x_offset, y_offset, &sprite_bytes);

        chip8.ram[0x200] = 0xD2;
        chip8.ram[0x201] = 0x34;
        chip8.ram[0x300..0x304].copy_from_slice(&sprite_bytes);
        chip8.set_index_register(0x300);
        chip8.set_variable_register(0x2, x_offset);
        chip8.set_variable_register(0x3, y_offset);

        chip8.execute_next_instruction();

        assert_eq!(test_frame_buffer, chip8.frame_buffer);
    }

    #[test]
    fn execute_next_instruction_can_execute_call_subroutine() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x22;
        chip8.ram[0x201] = 0x11;
        chip8.program_counter = 0x200;

        chip8.execute_next_instruction();

        assert_eq!(0x211, chip8.program_counter);
        assert_eq!(0x202, chip8.stack[0]);
        assert_eq!(1, chip8.stack_pointer);
    }

    #[test]
    fn execute_next_instruction_can_execute_return_from_subroutine() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x211] = 0x00;
        chip8.ram[0x212] = 0xEE;
        chip8.program_counter = 0x211;
        chip8.stack[0] = 0x200;
        chip8.stack_pointer = 1;

        chip8.execute_next_instruction();

        assert_eq!(0x200, chip8.program_counter);
        assert_eq!(0, chip8.stack_pointer);
    }

    #[test]
    fn execute_next_instruction_can_execute_skip_instruction_if_vx_equals_nn() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x33;
        chip8.ram[0x201] = 0x34;
        chip8.program_counter = 0x200;
        chip8.variable_registers[0x03] = 0x34;

        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_skip_instruction_if_vx_not_equals_nn() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x43;
        chip8.ram[0x201] = 0x34;
        chip8.program_counter = 0x200;
        chip8.variable_registers[0x03] = 0x35;

        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_skip_instruction_if_vx_equals_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x53;
        chip8.ram[0x201] = 0x20;
        chip8.program_counter = 0x200;
        chip8.variable_registers[0x02] = 0x34;
        chip8.variable_registers[0x03] = 0x34;

        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_skip_instruction_if_vx_not_equals_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.ram[0x200] = 0x93;
        chip8.ram[0x201] = 0x20;
        chip8.program_counter = 0x200;
        chip8.variable_registers[0x02] = 0x34;
        chip8.variable_registers[0x03] = 0x35;

        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_instruction_can_execute_set_vx_to_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xF0;
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xF] = 0xAF;

        chip8.execute_next_instruction();

        assert_eq!(0xAF, chip8.variable_registers[0x2]);
    }
    
    #[test]
    fn execute_instruction_can_execute_binary_or_vx_with_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xF] = 0xAF;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xF1;

        chip8.execute_next_instruction();

        assert_eq!(0x34 | 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn execute_instruction_can_execute_binary_and_vx_with_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xF] = 0xAF;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xF2;

        chip8.execute_next_instruction();

        assert_eq!(0x34 & 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn execute_instruction_can_execute_binary_xor_vx_with_vy() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xF] = 0xAF;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xF3;

        chip8.execute_next_instruction();

        assert_eq!(0x34 ^ 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn execute_instruction_can_execute_add_vy_to_vx() {
        let x_val: u8 = 0x34;
        let y_val: u8 = 0xAF;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xC4;

        chip8.execute_next_instruction();

        assert_eq!(x_val + y_val, chip8.variable_registers[0x2]);
        assert_eq!(0x0, chip8.variable_registers[0xF]);
    }

    #[test]
    fn execute_instruction_can_execute_subtract_vy_from_vx() {
        let x_val: u8 = 0xAF;
        let y_val: u8 = 0x34;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xC5;

        chip8.execute_next_instruction();

        assert_eq!(x_val.wrapping_sub(y_val), chip8.variable_registers[0x2]);
        assert_eq!(0x1, chip8.variable_registers[0xF]);
    }

    #[test]
    fn execute_instruction_can_execute_subtract_vx_from_vy_into_vx() {
        let x_val: u8 = 0x34;
        let y_val: u8 = 0xAF;
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xC7;

        chip8.execute_next_instruction();

        assert_eq!(y_val.wrapping_sub(x_val), chip8.variable_registers[0x2]);
        assert_eq!(0x1, chip8.variable_registers[0xF]);
    }

    #[test]
    fn execute_instruction_can_execute_shift_vx_right() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x22;
        chip8.variable_registers[0xC] = 0x34;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xC6;

        chip8.execute_next_instruction();

        assert_eq!(0x34 >> 1, chip8.variable_registers[0x2]);
    }

    #[test]
    fn execute_instruction_can_execute_shift_vx_left() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.variable_registers[0x2] = 0x22;
        chip8.variable_registers[0xC] = 0x34;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xCE;

        chip8.execute_next_instruction();

        assert_eq!(0x34 << 1, chip8.variable_registers[0x2]);
    }

    #[test]
    fn execute_instruction_can_execute_jump_with_offset() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xB2;
        chip8.ram[0x201] = 0x34;
        chip8.variable_registers[0x0] = 0x12;

        chip8.execute_next_instruction();

        assert_eq!(0x234 + 0x12, chip8.program_counter);
    }

    #[test]
    fn execute_instruction_can_execute_randomize_vx() {
        let rng = StdRng::seed_from_u64(0);
        let mut test_rng = rng.clone();
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rng);
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xC2;
        chip8.ram[0x201] = 0x34;

        chip8.execute_next_instruction();

        assert_eq!(test_rng.random::<u8>() & 0x34, chip8.variable_registers[0x2]);
    }

    #[test]
    fn execute_instruction_can_execute_skip_if_key_down() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x200;
        chip8.variable_registers[0x4] = Chip8Key::C.key_index() as u8;
        chip8.ram[0x200] = 0xE4;
        chip8.ram[0x201] = 0x9E;

        chip8.key_down(Chip8Key::C);
        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_instruction_can_execute_skip_if_key_up() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());
        chip8.program_counter = 0x200;
        chip8.variable_registers[0x4] = Chip8Key::C.key_index() as u8;
        chip8.ram[0x200] = 0xE4;
        chip8.ram[0x201] = 0xA1;

        chip8.key_up(Chip8Key::C);
        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_instruction_can_execute_set_vx_to_delay_timer() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x07;
        chip8.delay_timer = 0x34;

        chip8.execute_next_instruction();

        assert_eq!(0x34, chip8.variable_registers[0x3]);
    }

    #[test]
    fn execute_instruction_can_execute_set_delay_timer_to_vx() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x15;
        chip8.variable_registers[0x3] = 0x34;

        chip8.execute_next_instruction();

        assert_eq!(0x34, chip8.delay_timer);
    }

    #[test]
    fn execute_instruction_can_execute_set_sound_timer_to_vx() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x18;
        chip8.variable_registers[0x3] = 0x34;

        chip8.execute_next_instruction();

        assert_eq!(0x34, chip8.sound_timer);
    }

    #[test]
    fn execute_instruction_can_execute_add_vx_to_index_register() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x1E;
        chip8.variable_registers[0x3] = 0x34;
        chip8.index_register = 0x22;

        chip8.execute_next_instruction();

        assert_eq!(0x34 + 0x22, chip8.index_register);
    }

    #[test]
    fn execute_instruction_can_execute_put_key_int_vx() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x0A;
        chip8.key_down(Chip8Key::C);

        chip8.execute_next_instruction();

        chip8.key_up(Chip8Key::C);

        chip8.execute_next_instruction();

        assert_eq!(Chip8Key::C.key_index() as u8, chip8.variable_registers[0x3]);
    }

    #[test]
    fn execute_instruction_can_point_index_register_at_font_character() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x29;
        chip8.variable_registers[0x3] = 0x5;

        chip8.execute_next_instruction();

        assert_eq!(FONT_START_ADDRESS as u16 + (0x5 * 5), chip8.index_register);
    }

    #[test]
    fn execute_instruction_can_execute_put_vx_decimal_digits_into_memory() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x33;
        chip8.variable_registers[0x3] = 0x9C;
        chip8.index_register = 0x344;

        chip8.execute_next_instruction();

        assert_eq!(1, chip8.ram[0x344]);
        assert_eq!(5, chip8.ram[0x345]);
        assert_eq!(6, chip8.ram[0x346]);
    }

    #[test]
    fn execute_instruction_can_execute_store_variable_registers_to_memory() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x55;
        chip8.variable_registers[0x0] = 0x12;
        chip8.variable_registers[0x1] = 0x23;
        chip8.variable_registers[0x2] = 0x45;
        chip8.variable_registers[0x3] = 0x67;
        chip8.index_register = 0xC00;

        chip8.execute_next_instruction();

        assert_eq!(0x12, chip8.ram[0xC00]);
        assert_eq!(0x23, chip8.ram[0xC01]);
        assert_eq!(0x45, chip8.ram[0xC02]);
        assert_eq!(0x67, chip8.ram[0xC03]);
        assert_eq!(0xC04, chip8.index_register);
    }

    #[test]
    fn execute_instruction_can_execute_load_variable_registers_from_memory() {
        let mut chip8 = Chip8::new(EmulatorType::CosmacVip, rand::rng());

        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0xF3;
        chip8.ram[0x201] = 0x65;
        chip8.ram[0xC00] = 0x12;
        chip8.ram[0xC01] = 0x23;
        chip8.ram[0xC02] = 0x45;
        chip8.ram[0xC03] = 0x67;
        chip8.index_register = 0xC00;

        chip8.load_variable_registers_from_memory(0x3);

        assert_eq!(0x12, chip8.variable_registers[0x0]);
        assert_eq!(0x23, chip8.variable_registers[0x1]);
        assert_eq!(0x45, chip8.variable_registers[0x2]);
        assert_eq!(0x67, chip8.variable_registers[0x3]);
        assert_eq!(0xC04, chip8.index_register);
    }
}