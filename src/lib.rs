/// The emulators display width in pixels
pub const DISPLAY_WIDTH: usize = 64;

/// The emulators display height in pixels
pub const DISPLAY_HEIGHT: usize = 32;

const PROGRAM_START_ADDRESS: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const VARIABLE_REGISTER_COUNT: usize = 16;

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

/// Represents the set of Chip-8 instructions
#[derive(Debug, PartialEq, Eq)]
enum Chip8Instruction {
    /// Instruction to clear the screen (i.e. write all pixes as off)
    ClearScreen,
    /// Instruction to set program counter to the provided address
    Jump(u16),
    /// Instruction to set the variable register x to nn
    SetVariableRegister(u8, u8),
    /// Instruction to add the value of nn to variable register x
    AddToVariableRegister(u8, u8),
    /// Instruction to set the index register to nnn
    SetIndexRegister(u16),
    /// Instruction to draw an n pixel tall sprite from the memory location in the index register
    /// with horizontal coordinate in variable register x and vertical screen coordinate in
    /// variable register y
    Draw(u8, u8, u8),
    /// Instruction to push the current address to the stack and jump to a subroutine at a new address
    CallSubroutine(u16),
    /// Instruction to pop an address from the stack and return the program counter to that address
    ReturnFromSubroutine,
    /// Instruction to skip the next instruction if VX == NN
    SkipInstructionIfVxEqualsNn(u8, u8),
    /// Instruction to skip the next instruction if VX != NN
    SkipInstructionIfVxNotEqualsNn(u8, u8),
    /// Instruction to skip the next instruction if VX == VY
    SkipInstructionIfVxEqualsVy(u8, u8),
    /// Instruction to skip the next instruction if VX != VY
    SkipInstructionIfVxNotEqualsVy(u8, u8),
    /// Instruction to set Vx to the value of Vy
    SetVxToVy(u8, u8),
    /// Instruction to set Vx to the binary or of Vx and Vy
    BinaryOrVxWithVy(u8, u8),
    /// Instruction to set Vx to the binary and of Vx and Vy
    BinaryAndVxWithVy(u8, u8),
    /// Instruction to set Vx to the binary xor of Vx and Vy
    BinaryXorVxWithVy(u8, u8),
    /// Instruction to add Vy to Vx
    AddVyToVx(u8, u8),
    SubtractVyFromVx(u8, u8),
    SubtractVxFromVyIntoVx(u8, u8),
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

    /// Decodes the opcode as a CHIP-8 instruction
    fn as_instruction(&self) -> Result<Chip8Instruction, String> {
        match self.opcode {
            0x00E0 => Ok(Chip8Instruction::ClearScreen),
            0x00EE => Ok(Chip8Instruction::ReturnFromSubroutine),
            0x1000..=0x1FFF => Ok(Chip8Instruction::Jump(self.nnn())),
            0x2000..=0x2FFF => Ok(Chip8Instruction::CallSubroutine(self.nnn())),
            0x3000..=0x3FFF => Ok(Chip8Instruction::SkipInstructionIfVxEqualsNn(self.x(), self.nn())),
            0x4000..=0x4FFF => Ok(Chip8Instruction::SkipInstructionIfVxNotEqualsNn(self.x(), self.nn())),
            0x5000..=0x5FFF => Ok(Chip8Instruction::SkipInstructionIfVxEqualsVy(self.x(), self.y())),
            0x6000..=0x6FFF => Ok(Chip8Instruction::SetVariableRegister(self.x(), self.nn())),
            0x7000..=0x7FFF => Ok(Chip8Instruction::AddToVariableRegister(self.x(), self.nn())),
            0x8000..=0x8FFF => {
                match self.n() {
                    0x0 => Ok(Chip8Instruction::SetVxToVy(self.x(), self.y())),
                    0x1 => Ok(Chip8Instruction::BinaryOrVxWithVy(self.x(), self.y())),
                    0x2 => Ok(Chip8Instruction::BinaryAndVxWithVy(self.x(), self.y())),
                    0x3 => Ok(Chip8Instruction::BinaryXorVxWithVy(self.x(), self.y())),
                    0x4 => Ok(Chip8Instruction::AddVyToVx(self.x(), self.y())),
                    0x5 => Ok(Chip8Instruction::SubtractVyFromVx(self.x(), self.y())),
                    // 0x6 => todo!(),
                    0x7 => Ok(Chip8Instruction::SubtractVxFromVyIntoVx(self.x(), self.y())),
                    // 0xE => todo!(),
                    _ => Err(format!("Encountered invalid opcode {:X}", self.opcode)) // TODO test this case
                }
            },
            0x9000..=0x9FFF => Ok(Chip8Instruction::SkipInstructionIfVxNotEqualsVy(self.x(), self.y())),
            0xA000..=0xAFFF => Ok(Chip8Instruction::SetIndexRegister(self.nnn())),
            // 0xB000..=0xBFFF => todo!(),
            // 0xC000..=0xCFFF => todo!(),
            0xD000..=0xDFFF => Ok(Chip8Instruction::Draw(self.x(), self.y(), self.n())),
            0xE000..=0xEFFF => {
                match self.nn() {
                    // 0x9E => todo!(),
                    // 0xA1 => todo!(),
                    _ => Err(format!("Encountered invalid opcode {:X}", self.opcode)) // TODO test this case
                }
            }
            0xF000..=0xFFFF => {
                match self.nn() {
                    // 0x07 => todo!(),
                    // 0x0A => todo!(),
                    // 0x15 => todo!(),
                    // 0x18 => todo!(),
                    // 0x1E => todo!(),
                    // 0x29 => todo!(),
                    // 0x33 => todo!(),
                    // 0x55 => todo!(),
                    // 0x65 => todo!(),
                    _ => Err(format!("Encountered invalid opcode {:X}", self.opcode)) // TODO test this case
                }
            }
            _ => Err(format!("Encountered invalid opcode {:X}", self.opcode))
        }
    }
}

/// Represents a Chip8 interpreter
#[derive(Debug)]
pub struct Chip8 {
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
}

impl Default for Chip8 {
    /// Creates a default Chip8 instance
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {
    /// Creates a new Chip8 instance
    pub fn new() -> Self {
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
        };

        chip8.ram[0x050..0x050 + FONT.len()].copy_from_slice(&FONT);

        chip8
    }

    /// Gets the Chip-8 instances frame buffer
    pub fn frame_buffer(&self) -> &[u8; DISPLAY_WIDTH * DISPLAY_HEIGHT] {
        &self.frame_buffer
    }

    /// Loads program into memory starting at address `{PROGRAM_START_ADDRESS}`
    pub fn load_program(&mut self, program: &[u8]) {
        let start_address = PROGRAM_START_ADDRESS as usize;
        self.ram[start_address..start_address + program.len()].copy_from_slice(program);
    }

    /// Execute the next instruction at the address pointed to by the program counter register
    pub fn execute_next_instruction(&mut self) {
        // let instruction = self.fetch_next_instruction().unwrap_or_else(|err| panic!("{}", err));
        let fetch_result = self.fetch_next_instruction();
        self.program_counter += 2;

        if let Ok(instruction) = fetch_result {
            match instruction {
                Chip8Instruction::ClearScreen => self.clear_screen(),
                Chip8Instruction::Jump(nnn) => self.jump(nnn),
                Chip8Instruction::SetVariableRegister(x, nn) => self.set_variable_register(x, nn),
                Chip8Instruction::AddToVariableRegister(x, nn) => self.add_to_variable_register(x, nn),
                Chip8Instruction::SetIndexRegister(nnn) => self.set_index_register(nnn),
                Chip8Instruction::Draw(x, y, n) => self.draw(x, y, n),
                Chip8Instruction::CallSubroutine(nnn) => self.call_subroutine(nnn),
                Chip8Instruction::ReturnFromSubroutine => self.return_from_subroutine(),
                Chip8Instruction::SkipInstructionIfVxEqualsNn(x, nn) => self.skip_instruction_if_vx_equals_nn(x, nn),
                Chip8Instruction::SkipInstructionIfVxNotEqualsNn(x, nn) => self.skip_instruction_if_vx_not_equals_nn(x, nn),
                Chip8Instruction::SkipInstructionIfVxEqualsVy(x, y) => self.skip_instruction_if_vx_equals_vy(x, y),
                Chip8Instruction::SkipInstructionIfVxNotEqualsVy(x, y) => self.skip_instruction_if_vx_not_equals_vy(x, y),
                Chip8Instruction::SetVxToVy(x, y) => self.set_vx_to_vy(x, y),
                Chip8Instruction::BinaryOrVxWithVy(x, y) => self.binary_or_vx_with_vy(x, y),
                Chip8Instruction::BinaryAndVxWithVy(x, y) => self.binary_and_vx_with_vy(x, y),
                Chip8Instruction::BinaryXorVxWithVy(x, y) => self.binary_xor_vx_with_vy(x, y),
                Chip8Instruction::AddVyToVx(x, y) => self.add_vy_to_vx(x, y),
                Chip8Instruction::SubtractVyFromVx(x, y) => self.subtract_vy_from_vx(x, y),
                Chip8Instruction::SubtractVxFromVyIntoVx(x, y) => self.subtract_vx_from_vy_into_vx(x, y),
            };
        }
    }

    /// Fetch the next instruction at the address pointed to by the program counter register
    fn fetch_next_instruction(&mut self) -> Result<Chip8Instruction, String> {
        let high = self.ram[self.program_counter as usize];
        let low = self.ram[(self.program_counter + 1) as usize];

        let opcode = (high as u16) << 8 | low as u16;
        OpCode::new(opcode).as_instruction()
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

    /// draws an n pixel tall sprite from the memory location that the index register is holding to
    /// the frame buffer, at horizontal X coordinate held in variable register at index x and the Y
    /// coordinate held in the variable register at index y
    fn draw(&mut self, x: u8, y: u8, n: u8) {
        let sprite_memory_address = self.index_register as usize;
        let sprite_bytes = &self.ram[sprite_memory_address..sprite_memory_address + n as usize];
        let x_offset = self.variable_registers[x as usize] as usize % DISPLAY_WIDTH;
        let y_offset = self.variable_registers[y as usize] as usize % DISPLAY_HEIGHT;

        // reset the VF register. We will flip it if any pixes go from ON to OFF
        self.variable_registers[0xF] = 0;

        // iterate over the sprite bytes
        for (row, byte) in sprite_bytes.iter().enumerate() {
            // iterate over the bites in the current sprite byte
            for bit_index in 0..8 {
                let pixel_x_index = x_offset + bit_index;
                let pixel_y_index = y_offset + row;

                // Don't draw sprite pixels if they go off the edge of the screen
                if pixel_x_index < DISPLAY_WIDTH && pixel_y_index < DISPLAY_HEIGHT {
                    let sprite_pixel_value= (byte >> (7 - bit_index)) & 0x01;
                    let frame_buffer_pixel_index = pixel_y_index * DISPLAY_WIDTH + pixel_x_index;
                    let frame_buffer_pixel_value = self.frame_buffer[frame_buffer_pixel_index];

                    // If a pixes was on but is now off flip the VF register
                    if frame_buffer_pixel_value == 1 && sprite_pixel_value == 0 {
                        self.variable_registers[0xF] = 1;
                    }

                    self.frame_buffer[frame_buffer_pixel_index] = sprite_pixel_value;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw_test_sprite(x_offset: u8, y_offset: u8, sprite_bytes: &[u8]) -> [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT] {
        let mut test_frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        for (row, byte) in sprite_bytes.iter().enumerate() {
            for bit in 0..8 {
                let sprite_bit_value = (byte >> (7 - bit)) & 0x01;
                let x_index = (x_offset + bit) as usize;
                let y_index = y_offset as usize + row;

                if x_index < DISPLAY_WIDTH && y_index < DISPLAY_HEIGHT {
                    test_frame_buffer[y_index * DISPLAY_WIDTH + x_index] = sprite_bit_value;
                }
            }
        }
        test_frame_buffer
    }

    #[test]
    fn can_create_new_chip_8() {
        let chip8 = Chip8::new();
        Chip8::default();

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

        assert_eq!(expected_ram, chip8.ram);
        assert_eq!(expected_frame_buffer, chip8.frame_buffer);
        assert_eq!(expected_stack, chip8.stack);
        assert_eq!(expected_stack_pointer, chip8.stack_pointer);
        assert_eq!(expected_delay_timer, chip8.delay_timer);
        assert_eq!(expected_sound_timer, chip8.sound_timer);
        assert_eq!(expected_program_counter, chip8.program_counter);
        assert_eq!(expected_index_register, chip8.index_register);
        assert_eq!(expected_variable_registers, chip8.variable_registers);
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
    fn clear_screen_has_correct_opcode() {
        let opcode = OpCode::new(0x00E0);
        assert_eq!(Ok(Chip8Instruction::ClearScreen), opcode.as_instruction());
    }

    #[test]
    fn jump_has_correct_opcode() {
        let opcode = OpCode::new(0x1234);
        assert_eq!(Ok(Chip8Instruction::Jump(0x234)), opcode.as_instruction());
    }

    #[test]
    fn set_variable_register_has_correct_opcode() {
        let opcode = OpCode::new(0x6234);
        assert_eq!(Ok(Chip8Instruction::SetVariableRegister(0x2, 0x34)), opcode.as_instruction());
    }

    #[test]
    fn add_to_variable_register_has_correct_opcode() {
        let opcode = OpCode::new(0x7234);
        assert_eq!(Ok(Chip8Instruction::AddToVariableRegister(0x2, 0x34)), opcode.as_instruction());
    }

    #[test]
    fn set_index_register_has_correct_opcode() {
        let opcode = OpCode::new(0xA234);
        assert_eq!(Ok(Chip8Instruction::SetIndexRegister(0x234)), opcode.as_instruction());
    }

    #[test]
    fn draw_has_correct_opcode() {
        let opcode = OpCode::new(0xD234);
        assert_eq!(Ok(Chip8Instruction::Draw(0x2, 0x3, 0x4)), opcode.as_instruction());
    }

    #[test]
    fn invalid_opcode_returns_error_when_parsed_to_instruction() {
        assert!(OpCode::new(0x0234).as_instruction().is_err());
    }

    #[test]
    fn can_load_program() {
        let mut chip8 = Chip8::new();
        let program = [0x00, 0xE0, 0x12, 0x34, 0x56, 0x78];
        chip8.load_program(&program);

        let start_address = PROGRAM_START_ADDRESS as usize;
        assert_eq!(program, chip8.ram[start_address..start_address + program.len()]);
    }

    #[test]
    fn can_fetch_next_instruction() {
        let mut chip8 = Chip8::new();
        chip8.ram[0x200] = 0x00;
        chip8.ram[0x201] = 0xE0;
        chip8.program_counter = 0x200;

        let instruction = chip8.fetch_next_instruction();
        assert_eq!(Ok(Chip8Instruction::ClearScreen), instruction);
    }

    #[test]
    fn can_clear_screen() {
        let mut chip8 = Chip8::new();
        chip8.frame_buffer = [1; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        chip8.clear_screen();
        assert_eq!([0; DISPLAY_WIDTH * DISPLAY_HEIGHT], chip8.frame_buffer);
    }

    #[test]
    fn can_jump() {
        let mut chip8 = Chip8::new();
        chip8.program_counter = 0x200;
        chip8.jump(0x300);
        assert_eq!(0x300, chip8.program_counter);
    }

    #[test]
    fn can_set_variable_register() {
        let mut chip8 = Chip8::new();

        chip8.set_variable_register(0x2, 0x34);
        chip8.set_variable_register(0x7, 0xAA);

        assert_eq!(0x34, chip8.variable_registers[2]);
        assert_eq!(0xAA, chip8.variable_registers[7]);
    }

    #[test]
    fn can_add_to_variable_register() {
        let mut chip8 = Chip8::new();

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
        let mut chip8 = Chip8::new();

        chip8.set_variable_register(0x2, initial_value);

        assert_eq!(initial_value, chip8.variable_registers[2]);

        chip8.add_to_variable_register(0x2, value_to_add);

        assert_eq!(expected_result, chip8.variable_registers[2]);
    }

    #[test]
    fn can_set_index_register() {
        let mut chip8 = Chip8::new();
        chip8.set_index_register(0x300);
        assert_eq!(0x300, chip8.index_register);
    }

    #[test]
    fn can_draw() {
        let x_offset = 34;
        let y_offset = 12;
        let sprite_bytes = [0b11111111, 0b01010101, 0b00000000, 0b11011101];
        let expected_frame_buffer = draw_test_sprite(x_offset, y_offset, &sprite_bytes);
        let mut chip8 = Chip8::new();

        chip8.frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        chip8.ram[0x300..0x304].copy_from_slice(&sprite_bytes);
        chip8.set_index_register(0x300);
        chip8.set_variable_register(0x2, x_offset);
        chip8.set_variable_register(0x3, y_offset);

        chip8.draw(0x2, 0x3, 0x4);

        assert_eq!(expected_frame_buffer, chip8.frame_buffer);
    }

    #[test]
    fn drawing_sprites_near_edge_does_not_wrap() {
        let x_offset = 60;
        let y_offset = 30;
        let sprite_bytes = [0b11111111, 0b01010101, 0b00000000, 0b11011101];
        let expected_frame_buffer = draw_test_sprite(x_offset, y_offset, &sprite_bytes);
        let mut chip8 = Chip8::new();

        chip8.frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        chip8.ram[0x300..0x304].copy_from_slice(&sprite_bytes);
        chip8.set_index_register(0x300);
        chip8.set_variable_register(0x2, x_offset);
        chip8.set_variable_register(0x3, y_offset);

        chip8.draw(0x2, 0x3, 0x4);

        assert_eq!(expected_frame_buffer, chip8.frame_buffer);
    }

    #[test]
    fn can_call_subroutine() {
        let mut chip8 = Chip8::new();
        chip8.program_counter = 0x202;

        chip8.call_subroutine(0x300);

        assert_eq!(0x202, chip8.stack[0]);
        assert_eq!(1, chip8.stack_pointer);
        assert_eq!(0x300, chip8.program_counter);
    }

    #[test]
    fn can_return_from_subroutine() {
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_equals_nn(0x2, 0x34);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_equals_nn_does_not_skips_when_not_equal() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_equals_nn(0x2, 0x35);

        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_not_equals_nn_skips_when_not_equal() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_not_equals_nn(0x2, 0x35);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_not_equals_nn_does_not_skips_when_equal() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_not_equals_nn(0x2, 0x34);

        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_equals_vy_skips_when_equal() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0x3] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_equals_vy(0x2, 0x3);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_equals_vy_does_not_skips_when_not_equal() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0x3] = 0x35;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_equals_vy(0x2, 0x3);

        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_not_equals_vy_skips_when_not_equal() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0x3] = 0x35;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_not_equals_vy(0x2, 0x3);

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn skip_instruction_if_vx_not_equals_vy_does_not_skips_when_equal() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0x3] = 0x34;
        chip8.program_counter = 0x202;

        chip8.skip_instruction_if_vx_not_equals_vy(0x2, 0x3);

        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn can_set_vx_to_vy() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xF] = 0xAF;
        chip8.program_counter = 0x202;

        chip8.set_vx_to_vy(0x2, 0xF);

        assert_eq!(0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn can_binary_or_vx_with_vy() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xC] = 0xAF;

        chip8.binary_or_vx_with_vy(0x2, 0xC);

        assert_eq!(0x34 | 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn can_binary_and_vx_with_vy() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xC] = 0xAF;

        chip8.binary_and_vx_with_vy(0x2, 0xC);

        assert_eq!(0x34 & 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn can_binary_xor_vx_with_vy() {
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = 0x34;
        chip8.variable_registers[0xC] = 0xAF;

        chip8.binary_xor_vx_with_vy(0x2, 0xC);

        assert_eq!(0x34 ^ 0xAF, chip8.variable_registers[0x2]);
    }

    #[test]
    fn can_add_vy_to_vx() {
        let x_val: u8 = 0x34;
        let y_val: u8 = 0xAF;
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.variable_registers[0xF] = 0x01;

        chip8.subtract_vx_from_vy_into_vx(0x2, 0xC);

        assert_eq!(y_val.wrapping_sub(x_val), chip8.variable_registers[0x2]);
        assert_eq!(0x0, chip8.variable_registers[0xF]);
    }

    #[test]
    fn execute_next_instruction_can_execute_clear_screen() {
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
        chip8.ram[0x200] = 0x12;
        chip8.ram[0x201] = 0x34;
        chip8.program_counter = 0x200;

        chip8.execute_next_instruction();

        assert_eq!(0x234, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_set_variable_register() {
        let mut chip8 = Chip8::new();
        chip8.ram[0x200] = 0x63;
        chip8.ram[0x201] = 0xBC;
        chip8.program_counter = 0x200;

        chip8.execute_next_instruction();

        assert_eq!(0xBC, chip8.variable_registers[0x03]);
        assert_eq!(0x202, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_add_to_variable_register() {
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let expected_frame_buffer = draw_test_sprite(x_offset, y_offset, &sprite_bytes);
        let mut chip8 = Chip8::new();

        chip8.ram[0x200] = 0xD2;
        chip8.ram[0x201] = 0x34;
        chip8.ram[0x300..0x304].copy_from_slice(&sprite_bytes);
        chip8.set_index_register(0x300);
        chip8.set_variable_register(0x2, x_offset);
        chip8.set_variable_register(0x3, y_offset);

        chip8.execute_next_instruction();

        assert_eq!(expected_frame_buffer, chip8.frame_buffer);
    }

    #[test]
    fn execute_next_instruction_can_execute_call_subroutine() {
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
        chip8.ram[0x200] = 0x33;
        chip8.ram[0x201] = 0x34;
        chip8.program_counter = 0x202;
        chip8.variable_registers[0x03] = 0x34;

        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_skip_instruction_if_vx_not_equals_nn() {
        let mut chip8 = Chip8::new();
        chip8.ram[0x200] = 0x33;
        chip8.ram[0x201] = 0x34;
        chip8.program_counter = 0x202;
        chip8.variable_registers[0x03] = 0x35;

        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_skip_instruction_if_vx_equals_vy() {
        let mut chip8 = Chip8::new();
        chip8.ram[0x200] = 0x53;
        chip8.ram[0x201] = 0x20;
        chip8.program_counter = 0x202;
        chip8.variable_registers[0x02] = 0x34;
        chip8.variable_registers[0x03] = 0x34;

        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_next_instruction_can_execute_skip_instruction_if_vx_not_equals_vy() {
        let mut chip8 = Chip8::new();
        chip8.ram[0x200] = 0x93;
        chip8.ram[0x201] = 0x20;
        chip8.program_counter = 0x202;
        chip8.variable_registers[0x02] = 0x34;
        chip8.variable_registers[0x03] = 0x35;

        chip8.execute_next_instruction();

        assert_eq!(0x204, chip8.program_counter);
    }

    #[test]
    fn execute_instruction_can_execute_set_vx_to_vy() {
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();

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
        let mut chip8 = Chip8::new();
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
        let mut chip8 = Chip8::new();
        chip8.variable_registers[0x2] = x_val;
        chip8.variable_registers[0xC] = y_val;
        chip8.program_counter = 0x200;
        chip8.ram[0x200] = 0x82;
        chip8.ram[0x201] = 0xC7;   
        
        chip8.execute_next_instruction();
        
        assert_eq!(y_val.wrapping_sub(x_val), chip8.variable_registers[0x2]);
        assert_eq!(0x1, chip8.variable_registers[0xF]);
    }
}