use std::{collections::HashMap, io::Write, os::windows::prelude::FileExt};


fn main() {
    let mut args = std::env::args();
    let asm_filename = args.nth(1).unwrap();
    let out_filename = args.next().unwrap();

    let asm_code_raw = std::fs::read_to_string(asm_filename).unwrap();
    let mut out_file = std::fs::File::create(out_filename).unwrap();

    // pass 1, remove comments
    let mut asm_code_pass1 = String::new();
    let mut comment = false;
    for char in asm_code_raw.chars() {
        if char == ';' {
            comment = true;
        } else if comment {
            if char == '\n' {
                comment = false;
                asm_code_pass1.push('\n');
            }
        } else {
            asm_code_pass1.push(char);
        }
    }

    // pass 2, assemble
    type ByteOffset = usize;
    let mut found_labels: HashMap<String, ByteOffset> = HashMap::new();
    let mut mentioned_labels: HashMap<String, ByteOffset> = HashMap::new(); 
    let mut byte_offset = 0;

    for line in asm_code_pass1.lines() {
        let mut words = line.split_whitespace();

        if let Some(word) = words.next() {
            match word {
                "move" => {
                    byte_offset += move_instruction(&mut words, &mut out_file);
                }
                "read" => {
                    byte_offset += read_instruction(&mut words, &mut out_file);
                }
                "write" => {
                    byte_offset += write_instruction(&mut words, &mut out_file);
                }
                "push" => {
                    byte_offset += push_instruction(&mut words, &mut out_file);
                }
                "pop" => {
                    byte_offset += pop_instruction(&mut words, &mut out_file);
                }
                "jump" => {
                    jump_instruction(&mut mentioned_labels, &mut byte_offset, &mut words, &mut out_file);
                }
                "add" => {
                    byte_offset += add_instruction(&mut out_file);
                }
                "sub" => {
                    byte_offset += sub_instruction(&mut out_file);
                }
                "mul" => {
                    byte_offset += mul_instruction(&mut out_file);
                }
                "div" => {
                    byte_offset += div_instruction(&mut out_file);
                }
                "equal" => {
                    byte_offset += equal_instruction(&mut out_file);
                }
                "less" => {
                    byte_offset += less_instruction(&mut out_file);
                }
                "not" => {
                    byte_offset += not_instruction(&mut out_file);
                }
                "and" => {
                    byte_offset += and_instruction(&mut out_file);
                }
                "or" => {
                    byte_offset += or_instruction(&mut out_file);
                }
                "xor" => {
                    byte_offset += xor_instruction(&mut out_file);
                }
                "byte" => {
                    let value = words.next().unwrap().trim().parse::<u8>().unwrap();
                    out_file.write_all(&[
                        value
                    ]).unwrap();

                    byte_offset += 1;
                }
                "dbyte" => {
                    let value = words.next().unwrap().trim().parse::<u16>().unwrap();
                    let value_bytes = value.to_be_bytes();

                    out_file.write_all(&[
                        value_bytes[0],
                        value_bytes[1]
                    ]).unwrap();

                    byte_offset += 2;
                }
                "qbyte" => {
                    let value = words.next().unwrap().trim().parse::<u32>().unwrap();
                    let value_bytes = value.to_be_bytes();

                    out_file.write_all(&[
                        value_bytes[0],
                        value_bytes[1],
                        value_bytes[2],
                        value_bytes[3]
                    ]).unwrap();

                    byte_offset += 4;
                }
                "obyte" => {
                    let value = words.next().unwrap().trim().parse::<u64>().unwrap();
                    let value_bytes = value.to_be_bytes();

                    out_file.write_all(&[
                        value_bytes[0],
                        value_bytes[1],
                        value_bytes[2],
                        value_bytes[3],
                        value_bytes[4],
                        value_bytes[5],
                        value_bytes[6],
                        value_bytes[7]
                    ]).unwrap();

                    byte_offset += 8;
                }
                word => {
                    if word.get(0..1) == Some(":") {
                        found_labels.insert(word[1..].to_string(), byte_offset);
                    }
                }
            }
        }
    }

    // pass 3, overwrite label addresses
    for mentioned_label in mentioned_labels {
        let address = found_labels.get(&mentioned_label.0).expect(format!("label {} is used but never declared", mentioned_label.0).as_str());
        let address_bytes = address.to_be_bytes();

        out_file.seek_write(
            &address_bytes,
            mentioned_label.1 as u64
        ).unwrap();
    }
}

fn register_name_to_index(register_name: &str) -> Option<u8> {
    match register_name {
        // registers
        "a" => Some(0),
        "b" => Some(1),
        "c" => Some(2),
        "d" => Some(3),
        "e" => Some(4),
        "f" => Some(5),
        "g" => Some(6),
        "h" => Some(7),
        "i" => Some(8),
        "j" => Some(9),
        "k" => Some(10),
        "l" => Some(11),
        "m" => Some(12),
        "n" => Some(13),
        // registers
        // special ones
        "pc" => Some(14), // program counter
        "sp" => Some(15), // stack pointer
        _ => {None}
    }
}

fn type_name_to_index(type_name: &str) -> Option<u8> {
    match type_name {
        "byte" => Some(0),
        "dbyte" => Some(1),
        "qbyte" => Some(2),
        "obyte" => Some(3),
        _ => {None}
    }
}

fn move_instruction<'a>(words: &mut impl Iterator<Item = &'a str>, out_file: &mut dyn Write) -> usize {
    let type_or_register = words.next().unwrap();
                
    if let Some(input_register) = register_name_to_index(type_or_register) {
        let output_register = words.next().unwrap();
        let output_register = register_name_to_index(output_register).unwrap();

        let io_registers = input_register << 4;
        let io_registers = io_registers | output_register;

        out_file.write_all(&[
            1, // move
            io_registers
        ]).unwrap();

        return 3;
    }

    let register = words.next().unwrap();
    let register = register_name_to_index(register).unwrap();

    match type_or_register {
        "byte" => {
            let value = words.next().unwrap();
            if let Ok(value) = value.parse::<u8>() {
                out_file.write_all(&[
                    2, // move
                    0, // byte
                    register,
                    value
                ]).unwrap();
            } else { // value is a char
                let value = value.chars().next().unwrap() as u8;
                out_file.write_all(&[
                    2, // move
                    0, // byte
                    register,
                    value
                ]).unwrap();
            }


            return 4;
        }
        "dbyte" => {
            let value = words.next().unwrap();
            let value = value.parse::<u16>().unwrap();
            let value_bytes = value.to_be_bytes();

            out_file.write_all(&[
                2, // move
                1, // dbyte
                register,
                value_bytes[0],
                value_bytes[1]
            ]).unwrap();

            return 5;
        }
        "qbyte" => {
            let value = words.next().unwrap();
            let value = value.parse::<u32>().unwrap();
            let value_bytes = value.to_be_bytes();

            out_file.write_all(&[
                2, // move
                2, // qbyte
                register,
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3]
            ]).unwrap();

            return 7;
        }
        "obyte" => {
            let value = words.next().unwrap();
            let value = value.parse::<u64>().unwrap();
            let value_bytes = value.to_be_bytes();

            out_file.write_all(&[
                2, // move
                3, // obyte
                register,
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
                value_bytes[4],
                value_bytes[5],
                value_bytes[6],
                value_bytes[7]
            ]).unwrap();

            return 11;
        }
        _ => {unreachable!()}
    }
}

fn read_instruction<'a>(words: &mut impl Iterator<Item = &'a str>, out_file: &mut dyn Write) -> usize {
    let value_type = words.next().unwrap();

    let input_register = words.next().unwrap();
    let input_register = register_name_to_index(input_register).unwrap();
    let register_or_value = words.next().unwrap();
    if let Some(address_register) = register_name_to_index(register_or_value) {
        let mut registers = input_register << 4;
        registers |= address_register;
        
        let value_type = type_name_to_index(value_type).unwrap();

        out_file.write_all(&[
            4, // read
            value_type,
            registers,
        ]).unwrap();

        3
    } else {
        let address = register_or_value.trim().parse::<u64>().unwrap();
        let address_bytes = address.to_be_bytes();

        let value_type = type_name_to_index(value_type).unwrap();

        out_file.write_all(&[
            3, // read
            value_type,
            input_register,
            address_bytes[0],
            address_bytes[1],
            address_bytes[2],
            address_bytes[3],
            address_bytes[4],
            address_bytes[5],
            address_bytes[6],
            address_bytes[7]
        ]).unwrap();

        11
    }
}

fn write_instruction<'a>(words: &mut impl Iterator<Item = &'a str>, out_file: &mut dyn Write) -> usize {
    let specified_type = words.next().unwrap();
    let specified_type = type_name_to_index(specified_type).unwrap();
    
    let output_register = words.next().unwrap();
    let output_register = register_name_to_index(output_register).unwrap();

    let register_or_address = words.next().unwrap();
    if let Some(address_register) = register_name_to_index(register_or_address) {
        let mut registers = output_register << 4;
        registers |= address_register;

        out_file.write_all(&[
            6, // write
            specified_type,
            registers
        ]).unwrap();

        3
    } else {
        let address = register_or_address.trim().parse::<u64>().unwrap();
        let address_bytes = address.to_be_bytes();

        out_file.write_all(&[
            5, // write
            specified_type,
            output_register,
            address_bytes[0],
            address_bytes[1],
            address_bytes[2],
            address_bytes[3],
            address_bytes[4],
            address_bytes[5],
            address_bytes[6],
            address_bytes[7]
        ]).unwrap();

        11
    }
}

fn push_instruction<'a>(words: &mut impl Iterator<Item = &'a str>, out_file: &mut dyn Write) -> usize {
    let specified_type = words.next().unwrap();

    let register_or_value = words.next().unwrap();

    if let Some(register) = register_name_to_index(register_or_value) {
        out_file.write_all(&[
            7, // push
            type_name_to_index(specified_type).unwrap(),
            register,
        ]).unwrap();

        3
    } else {
        match specified_type {
            "byte" => {
                let value = register_or_value.trim().parse::<u8>().unwrap();

                out_file.write_all(&[
                    8, // push
                    0, // byte
                    value
                ]).unwrap();

                3
            }
            "dbyte" => {
                let value = register_or_value.trim().parse::<u16>().unwrap();
                let value_bytes = value.to_be_bytes();

                out_file.write_all(&[
                    8, // push
                    1, // dbyte
                    value_bytes[0],
                    value_bytes[1]
                ]).unwrap();

                4
            }
            "qbyte" => {
                let value = register_or_value.trim().parse::<u32>().unwrap();
                let value_bytes = value.to_be_bytes();

                out_file.write_all(&[
                    8, // push
                    2, // qbyte
                    value_bytes[0],
                    value_bytes[1],
                    value_bytes[2],
                    value_bytes[3]
                ]).unwrap();

                6
            }
            "obyte" => {
                let value = register_or_value.trim().parse::<u64>().unwrap();
                let value_bytes = value.to_be_bytes();

                out_file.write_all(&[
                    8, // push
                    3, // obyte
                    value_bytes[0],
                    value_bytes[1],
                    value_bytes[2],
                    value_bytes[3],
                    value_bytes[4],
                    value_bytes[5],
                    value_bytes[6],
                    value_bytes[7]
                ]).unwrap();

                10
            }
            _ => {unreachable!()}
        }
    }
}

fn pop_instruction<'a>(words: &mut impl Iterator<Item = &'a str>, out_file: &mut dyn Write) -> usize {
    let specified_type = words.next().unwrap();

    let register = words.next().unwrap();

    out_file.write_all(&[
        9, // pop
        type_name_to_index(specified_type).unwrap(),
        register_name_to_index(register).unwrap(),
    ]).unwrap();

    3
}

fn jump_instruction<'a>(
    mentioned_labels: &mut HashMap<String, usize>,
    byte_offset: &mut usize,
    words: &mut impl Iterator<Item = &'a str>,
    out_file: &mut dyn Write
) {
    let next_word = words.next().unwrap();

    if let Ok(address) = next_word.trim().parse::<u64>() {
        let address_bytes = address.to_be_bytes();

        if let Some(condition) = words.next() {
            let condition = condition.trim().parse::<bool>().unwrap();
            let condition = condition as u8;

            out_file.write_all(&[
                12, // condtion jump
                condition,
                address_bytes[0],
                address_bytes[1],
                address_bytes[2],
                address_bytes[3],
                address_bytes[4],
                address_bytes[5],
                address_bytes[6],
                address_bytes[7]
            ]).unwrap();

            *byte_offset += 10;
        } else {
            out_file.write_all(&[
                10, // jump
                address_bytes[0],
                address_bytes[1],
                address_bytes[2],
                address_bytes[3],
                address_bytes[4],
                address_bytes[5],
                address_bytes[6],
                address_bytes[7]
            ]).unwrap();
        }

        *byte_offset += 9;
    } else if let Some(register) = register_name_to_index(next_word) {
        if let Some(condition) = words.next() {
            let condition = condition.trim().parse::<bool>().unwrap();
            let condition = condition as u8;

            out_file.write_all(&[
                13, // condition jump register
                condition,
                register
            ]).unwrap();

            *byte_offset += 3;
        } else {
            out_file.write_all(&[
                11, // jump register
                register
            ]).unwrap();
    
            *byte_offset += 2;
        }
    } else {
        if let Some(condition) = words.next() {
            let condition = condition.trim().parse::<bool>().unwrap();
            let condition = condition as u8;

            out_file.write_all(&[
                12, // condtion jump
                condition,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0
            ]).unwrap();

            mentioned_labels.insert(next_word.trim().to_string(), *byte_offset + 2);

            *byte_offset += 10;
        } else {
            out_file.write_all(&[
                10, // jump
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0
            ]).unwrap();
    
            mentioned_labels.insert(next_word.trim().to_string(), *byte_offset + 1);
    
            *byte_offset += 9;
        }
    }
}

fn add_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        14 // add
    ]).unwrap();
    
    1
}

fn sub_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        15 // sub
    ]).unwrap();
    
    1
}

fn mul_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        16 // mul
    ]).unwrap();

    1
}

fn div_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        17 // div
    ]).unwrap();

    1
}

fn equal_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        18 // equal
    ]).unwrap();

    1
}

fn less_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        19 // less
    ]).unwrap();

    1
}

fn not_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        20 // not
    ]).unwrap();

    1
}

fn and_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        21 // and
    ]).unwrap();

    1
}

fn or_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        22 // or
    ]).unwrap();

    1
}

fn xor_instruction(out_file: &mut dyn Write) -> usize {
    out_file.write_all(&[
        23 // xor
    ]).unwrap();

    1
}
