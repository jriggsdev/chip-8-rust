/// The emulators display width in pixels
const DISPLAY_WIDTH: usize = 64;

/// The emulators display height in pixels
const DISPLAY_HEIGHT: usize = 32;

/// The font sprite data consisting of hexadecimal numbers 0-F
const FONT: [u8; 80] = [
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
    fn as_instruction(&self) -> Chip8Instruction {
        match self.opcode {
            0x00E0 => Chip8Instruction::ClearScreen,
            0x1000..=0x1FFF => Chip8Instruction::Jump(self.nnn()),
            0x6000..=0x6FFF => Chip8Instruction::SetVariableRegister(self.x(), self.nn()),
            0x7000..=0x7FFF => Chip8Instruction::AddToVariableRegister(self.x(), self.nn()),
            0xA000..=0xAFFF => Chip8Instruction::SetIndexRegister(self.nnn()),
            0xD000..=0xDFFF => Chip8Instruction::Draw(self.x(), self.y(), self.n()),
            _ => panic!("Encountered invalid opcode {:X}", self.opcode)
        }
    }
}

/// Represents a Chip8 interpreter
#[derive(Debug)]
pub struct Chip8 {
    // 4096 bytes of memory
    ram: [u8; 4096],
    /// frame buffer for drawing screen
    frame_buffer: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    /// A stack of 16 2-byte addresses
    stack: [u16; 16],
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
    /// 16 8-bit variable registers
    variable_registers: [u8; 16],
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
            ram: [0; 4096],
            frame_buffer: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0,
            index_register: 0,
            variable_registers: [0; 16],
        };

        chip8.ram[0x050..0x050 + FONT.len()].copy_from_slice(&FONT);

        chip8
    }

    /// Execute the next instruction at the address pointed to by the program counter register
    pub fn execute_next_instruction(&mut self) {
        let instruction = self.fetch_next_instruction();
        todo!();
    }

    /// Fetch the next instruction at the address pointed to by the program counter register
    fn fetch_next_instruction(&mut self) -> Chip8Instruction {
        let high = self.ram[self.program_counter as usize];
        let low = self.ram[(self.program_counter + 1) as usize];

        let opcode = (high as u16) << 8 | low as u16;
        OpCode::new(opcode).as_instruction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_chip_8() {
        let chip8 = Chip8::new();
        Chip8::default();

        let mut expected_ram = [0; 4096];
        expected_ram[0x050..0x050 + FONT.len()].copy_from_slice(&FONT);

        let expected_frame_buffer = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        let expected_stack = [0; 16];
        let expected_stack_pointer = 0;
        let expected_delay_timer = 0;
        let expected_sound_timer = 0;
        let expected_program_counter = 0;
        let expected_index_register = 0;
        let expected_variable_registers = [0; 16];

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
        assert_eq!(Chip8Instruction::ClearScreen, opcode.as_instruction());
    }

    #[test]
    fn jump_has_correct_opcode() {
        let opcode = OpCode::new(0x1234);
        assert_eq!(Chip8Instruction::Jump(0x234), opcode.as_instruction());
    }

    #[test]
    fn set_variable_register_has_correct_opcode() {
        let opcode = OpCode::new(0x6234);
        assert_eq!(Chip8Instruction::SetVariableRegister(0x2, 0x34), opcode.as_instruction());
    }

    #[test]
    fn add_to_variable_register_has_correct_opcode() {
        let opcode = OpCode::new(0x7234);
        assert_eq!(Chip8Instruction::AddToVariableRegister(0x2, 0x34), opcode.as_instruction());
    }

    #[test]
    fn set_index_register_has_correct_opcode() {
        let opcode = OpCode::new(0xA234);
        assert_eq!(Chip8Instruction::SetIndexRegister(0x234), opcode.as_instruction());
    }

    #[test]
    fn draw_has_correct_opcode() {
        let opcode = OpCode::new(0xD234);
        assert_eq!(Chip8Instruction::Draw(0x2, 0x3, 0x4), opcode.as_instruction());
    }

    #[test]
    #[should_panic(expected = "Encountered invalid opcode 234")]
    fn invalid_opcode_panics_when_trying_to_get_as_instruction() {
        OpCode::new(0x0234).as_instruction();
    }
}