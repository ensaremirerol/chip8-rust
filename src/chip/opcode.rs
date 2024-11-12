#[derive(Debug, PartialEq)]
pub enum OpCode {
    CLS,               // Clear the display
    RET,               // Return from a subroutine
    SYSADDR(u16),      // Jump to a machine code routine at nnn
    JP(u16),           // Jump to location nnn
    CALL(u16),         // Call subroutine at nnn
    SEVxByte(u8, u8),  // Skip next instruction if Vx = kk
    SNEVxByte(u8, u8), // Skip next instruction if Vx != kk
    SEVxVy(u8, u8),    // Skip next instruction if Vx = Vy
    LDVxByte(u8, u8),  // Set Vx = kk
    ADDVxByte(u8, u8), // Set Vx = Vx + kk
    LDVxVy(u8, u8),    // Set Vx = Vy
    ORVxVy(u8, u8),    // Set Vx = Vx OR Vy
    ANDVxVy(u8, u8),   // Set Vx = Vx AND Vy
    XORVxVy(u8, u8),   // Set Vx = Vx XOR Vy
    ADDVxVy(u8, u8),   // Set Vx = Vx + Vy, set VF = carry
    SUBVxVy(u8, u8),   // Set Vx = Vx - Vy, set VF = NOT borrow
    SHRVyVx(u8, u8),   // Set Vx = Vy SHR 1
    SUBNVyVx(u8, u8),  // Set Vx = Vy - Vx, set VF = NOT borrow
    SHLVyVx(u8, u8),   // Set Vx = Vy SHL 1
    SNEVxVy(u8, u8),   // Skip next instruction if Vx != Vy
    LDI(u16),          // Set I = nnn
    JP0(u16),          // Jump to location nnn + V0
    RND(u8, u8),       // Set Vx = random byte AND kk
    DRW(u8, u8, u8), // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    SKPVx(u8),       // Skip next instruction if key with the value of Vx is pressed
    SKNPVx(u8),      // Skip next instruction if key with the value of Vx is not pressed
    LDDTVx(u8),      // Set Vx = delay timer value
    LDVxK(u8),       // Wait for a key press, store the value of the key in Vx
    LDVxDT(u8),      // Set delay timer = Vx
    LDSTVx(u8),      // Set sound timer = Vx
    ADDIVx(u8),      // Set I = I + Vx
    LDFVx(u8),       // Set I = location of sprite for digit Vx
    LDBVx(u8),       // Store BCD representation of Vx in memory locations I, I+1, and I+2
    LDIVx(u8),       // Store registers V0 through Vx in memory starting at location I
    LDVxI(u8),       // Read registers V0 through Vx from memory starting at location I
    SCD(u8),         // Scroll down n lines
    SCR,             // Scroll right 4 pixels
    SCL,             // Scroll left 4 pixels
    EXIT,            // Exit interpreter
    LOW,             // Disable extended screen mode
    HIGH,            // Enable extended screen mode
    DRWVxVy0(u8, u8), // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    LDHFVx(u8),       // Set I = location of sprite for digit Vx
    LDRVx(u8),        // Store registers V0 through Vx in memory starting at location I
    LDVxR(u8),        // Read registers V0 through Vx from memory starting at location I
}
