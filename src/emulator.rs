pub const RAM_SIZE: usize = 320_000;
pub const COUNTER_REG: usize = 14;
pub const STACK_REG: usize = 15;

// BYTE = 8 bits
// DBYTE = Double byte = 16 bits
// QBYTE = Quad byte = 32 bits
// OBYTE = Octal byte = 64 bits

pub struct Emulator {
    registers: [u64; 16],
    ram: [u8; RAM_SIZE]
}

impl Emulator {
    pub fn new(bin: &[u8]) -> Emulator {
        let mut ram = [0; RAM_SIZE];

        for (i, byte) in bin.iter().enumerate() {
            ram[i] = *byte;
        }

        let mut registers = [0; 16];
        registers[STACK_REG] = ram.len() as u64 - 1;

        Emulator {
            registers,
            ram
        }
    }

    fn read_byte(&self, addr: usize) -> u8 {
        self.ram[addr]
    }

    fn read_dbyte(&self, addr: usize) -> u16 {
        u16::from_be_bytes([
            self.ram[addr],
            self.ram[addr + 1]
        ])
    }

    fn read_qbyte(&self, addr: usize) -> u32 {
        u32::from_be_bytes([
            self.ram[addr],
            self.ram[addr + 1],
            self.ram[addr + 2],
            self.ram[addr + 3]
        ])
    }

    fn read_obyte(&self, addr: usize) -> u64 {
        u64::from_be_bytes([
            self.ram[addr],
            self.ram[addr + 1],
            self.ram[addr + 2],
            self.ram[addr + 3],
            self.ram[addr + 4],
            self.ram[addr + 5],
            self.ram[addr + 6],
            self.ram[addr + 7]
        ])
    }

    fn read_next_byte(&mut self) -> u8 {
        let byte = self.ram[self.registers[COUNTER_REG] as usize];
        self.registers[COUNTER_REG] += 1;
        byte
    }

    fn read_next_dbyte(&mut self) -> u16 {
        let byte1 = self.ram[self.registers[COUNTER_REG] as usize];
        let byte2 = self.ram[self.registers[COUNTER_REG] as usize + 1];
        self.registers[COUNTER_REG] += 2;
        u16::from_be_bytes([byte1, byte2])
    }

    fn read_next_qbyte(&mut self) -> u32 {
        let byte1 = self.ram[self.registers[COUNTER_REG] as usize];
        let byte2 = self.ram[self.registers[COUNTER_REG] as usize + 1];
        let byte3 = self.ram[self.registers[COUNTER_REG] as usize + 2];
        let byte4 = self.ram[self.registers[COUNTER_REG] as usize + 3];
        self.registers[COUNTER_REG] += 4;
        u32::from_be_bytes([byte1, byte2, byte3, byte4])
    }

    fn read_next_obyte(&mut self) -> u64 {
        let byte1 = self.ram[self.registers[COUNTER_REG] as usize];
        let byte2 = self.ram[self.registers[COUNTER_REG] as usize + 1];
        let byte3 = self.ram[self.registers[COUNTER_REG] as usize + 2];
        let byte4 = self.ram[self.registers[COUNTER_REG] as usize + 3];
        let byte5 = self.ram[self.registers[COUNTER_REG] as usize + 4];
        let byte6 = self.ram[self.registers[COUNTER_REG] as usize + 5];
        let byte7 = self.ram[self.registers[COUNTER_REG] as usize + 6];
        let byte8 = self.ram[self.registers[COUNTER_REG] as usize + 7];
        self.registers[COUNTER_REG] += 8;
        u64::from_be_bytes([byte1, byte2, byte3, byte4, byte5, byte6, byte7, byte8])
    }

    fn write_byte(&mut self, addr: usize, byte: u8) {
        self.ram[addr] = byte;
    }

    fn write_dbyte(&mut self, addr: usize, dbyte: u16) {
        let bytes = dbyte.to_be_bytes();

        self.ram[addr] = bytes[0];
        self.ram[addr + 1] = bytes[1];
    }

    fn write_qbyte(&mut self, addr: usize, qbyte: u32) {
        let bytes = qbyte.to_be_bytes();

        self.ram[addr] = bytes[0];
        self.ram[addr + 1] = bytes[1];
        self.ram[addr + 2] = bytes[2];
        self.ram[addr + 3] = bytes[3];
    }

    fn write_obyte(&mut self, addr: usize, obyte: u64) {
        let bytes = obyte.to_be_bytes();

        self.ram[addr] = bytes[0];
        self.ram[addr + 1] = bytes[1];
        self.ram[addr + 2] = bytes[2];
        self.ram[addr + 3] = bytes[3];
        self.ram[addr + 4] = bytes[4];
        self.ram[addr + 5] = bytes[5];
        self.ram[addr + 6] = bytes[6];
        self.ram[addr + 7] = bytes[7];
    }

    /// Push a value onto the stack
    /// 
    /// `param` - value to push
    /// 
    /// `bytes` - number of bytes to push
    fn push(&mut self, value: u64, bytes: usize) {
        let value_bytes = value.to_le_bytes();

        let value_offset = self.registers[STACK_REG] as usize - bytes + 1;

        for i in 0..bytes {
            self.ram[value_offset + i] = value_bytes[i];
        }

        self.registers[STACK_REG] -= bytes as u64;
    }

    fn pop(&mut self, bytes: usize) -> u64 {
        let mut bytes_read = [0; 8];

        for i in 0..bytes {
            bytes_read[i] = self.ram[self.registers[STACK_REG] as usize + i + 1];
        }

        self.registers[STACK_REG] += bytes as u64;

        u64::from_le_bytes(bytes_read)
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.read_next_byte();

            match instruction {
                0 => {}
                // move <register> <register>
                1 => {
                    let io_registers = self.read_next_byte();
                    let input_register = (io_registers & 0b1111_0000) >> 4;
                    let output_register = io_registers & 0b0000_1111;

                    self.registers[input_register as usize] = self.registers[output_register as usize];
                }
                // move <type> <register> <value>
                2 => {
                    let specified_type = self.read_next_byte();
                    let register = self.read_next_byte() as usize;
                    let value = match specified_type {
                        0 => self.read_next_byte() as u64,
                        1 => self.read_next_dbyte() as u64,
                        2 => self.read_next_qbyte() as u64,
                        3 => self.read_next_obyte() as u64,
                        _ => {unreachable!()}
                    };

                    self.registers[register] = value;
                }
                // read <type> <register> <address>
                3 => {
                    let specified_type = self.read_next_byte();
                    let register = self.read_next_byte() as usize;
                    let address = self.read_next_obyte() as usize;

                    self.registers[register] = match specified_type {
                        0 => self.read_byte(address) as u64,
                        1 => self.read_dbyte(address) as u64,
                        2 => self.read_qbyte(address) as u64,
                        3 => self.read_obyte(address) as u64,
                        _ => {unreachable!()}
                    }
                }
                // read <type> <register> <register>
                4 => {
                    let specified_type = self.read_next_byte();
                    let registers = self.read_next_byte() as usize;
                    let input_register = (registers & 0b1111_0000) >> 4;
                    let address_register = (registers & 0b0000_1111) as usize;

                    match specified_type {
                        0 => {
                            self.registers[input_register] = self.read_byte(self.registers[address_register] as usize) as u64;
                        }
                        1 => {
                            self.registers[input_register] = self.read_dbyte(self.registers[address_register] as usize) as u64;
                        }
                        2 => {
                            self.registers[input_register] = self.read_qbyte(self.registers[address_register] as usize) as u64;
                        }
                        3 => {
                            self.registers[input_register] = self.read_obyte(self.registers[address_register] as usize) as u64;
                        }
                        _ => {}
                    }
                }
                // write <type> <register> <address>
                5 => {
                    let specified_type = self.read_next_byte();
                    let register = self.read_next_byte() as usize;
                    let address = self.read_next_obyte() as usize;

                    match specified_type {
                        0 => {
                            self.write_byte(address, self.registers[register] as u8);
                        }
                        1 => {
                            self.write_dbyte(address, self.registers[register] as u16);
                        }
                        2 => {
                            self.write_qbyte(address, self.registers[register] as u32);
                        }
                        3 => {
                            self.write_obyte(address, self.registers[register] as u64);
                        }
                        _ => {}
                    }
                }
                // write <type> <register> <register>
                6 => {
                    let specified_type = self.read_next_byte();
                    let registers = self.read_next_byte();
                    let output_register = ((registers & 0b1111_0000) >> 4) as usize;
                    let address_register = (registers & 0b0000_1111) as usize;

                    match specified_type {
                        0 => {
                            self.write_byte(self.registers[address_register] as usize, self.registers[output_register] as u8);
                        }
                        1 => {
                            self.write_dbyte(self.registers[address_register] as usize, self.registers[output_register] as u16);
                        }
                        2 => {
                            self.write_qbyte(self.registers[address_register] as usize, self.registers[output_register] as u32);
                        }
                        3 => {
                            self.write_obyte(self.registers[address_register] as usize, self.registers[output_register] as u64);
                        }
                        _ => {}
                    }
                }
                // push <type> <register>
                7 => {
                    let specified_type = self.read_next_byte();
                    let register = self.read_next_byte() as usize;

                    match specified_type {
                        0 => {
                            self.push(self.registers[register], 1);
                        }
                        1 => {
                            self.push(self.registers[register], 2);
                        }
                        2 => {
                            self.push(self.registers[register], 4);
                        }
                        3 => {
                            self.push(self.registers[register], 8);
                        }
                        _ => {}
                    }
                }
                // push <type> <value>
                8 => {
                    let specified_type = self.read_next_byte();

                    match specified_type {
                        0 => {
                            let value = self.read_next_byte() as u64;
                            self.push(value, 1);
                        }
                        1 => {
                            let value = self.read_next_dbyte() as u64;
                            self.push(value, 2);
                        }
                        2 => {
                            let value = self.read_next_qbyte() as u64;
                            self.push(value, 4);
                        }
                        3 => {
                            let value = self.read_next_obyte() as u64;
                            self.push(value, 8);
                        }
                        _ => {}
                    }
                }
                // pop <type> <register>
                9 => {
                    let specified_type = self.read_next_byte();
                    let register = self.read_next_byte() as usize;

                    match specified_type {
                        0 => {
                            let value = self.pop(1);
                            self.registers[register] = value;
                        }
                        1 => {
                            let value = self.pop(2);
                            self.registers[register] = value;
                        }
                        2 => {
                            let value = self.pop(4);
                            self.registers[register] = value;
                        }
                        3 => {
                            let value = self.pop(8);
                            self.registers[register] = value;
                        }
                        _ => {}
                    }
                }
                // jump <address/label> technically label is an address
                10 => {
                    let address = self.read_next_obyte() as u64;

                    self.registers[COUNTER_REG] = address;
                }
                // jump <register>
                11 => {
                    let register = self.read_next_byte() as usize;

                    self.registers[COUNTER_REG] = self.registers[register];
                }
                // jump <address/label> <condition>
                12 => {
                    let condition = self.read_next_byte();

                    let address = self.read_next_obyte() as u64;
                    
                    if condition as u64 == self.registers[2] {
                        self.registers[COUNTER_REG] = address;
                    }
                }
                // jump <register> <condition>
                13 => {
                    let condition = self.read_next_byte();

                    let register = self.read_next_byte() as usize;
                    
                    if condition as u64 == self.registers[2] {
                        self.registers[COUNTER_REG] = self.registers[register as usize];
                    }
                }
                // add
                14 => {
                    self.registers[2] = self.registers[0] + self.registers[1];
                }
                // sub
                15 => {
                    self.registers[2] = self.registers[0] - self.registers[1];
                }
                // mul
                16 => {
                    self.registers[2] = self.registers[0] * self.registers[1];
                }
                // div
                17 => {
                    self.registers[2] = self.registers[0] / self.registers[1];
                    self.registers[3] = self.registers[0] % self.registers[1];
                }
                // equal
                18 => {
                    self.registers[2] = (self.registers[0] == self.registers[1]) as u64;
                }
                // less
                19 => {
                    self.registers[2] = (self.registers[0] < self.registers[1]) as u64;
                }
                // not
                20 => {
                    self.registers[2] = !self.registers[0];
                }
                // and
                21 => {
                    self.registers[2] = self.registers[0] & self.registers[1];
                }
                // or
                22 => {
                    self.registers[2] = self.registers[0] | self.registers[1];
                }
                // xor
                23 => {
                    self.registers[2] = self.registers[0] ^ self.registers[1];
                }
                _ => {}
            }
            
            println!("a: {}, f: {}", self.registers[0], self.registers[5]);

            if self.registers[COUNTER_REG] > 50 {
                break;
            }
        }
    }
}
