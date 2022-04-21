use std::collections::HashMap;

// Used for program.mif and dmemory.mif
// PROGram RAM Memory Initialization File
// Data MEMORY Memory Initialization File
const TOP_PROGRAM_COMMENT : &str = "MIPS Instruction Memory Initialization File";
const TOP_DATA_COMMENT : &str = "MIPS Data Memory Initialization File";
const DEPTH : u32 = 256;
const WIDTH : u32 = 32;
const ADDRESS_RADIX : &str = "HEX";
const DATA_RADIX : &str = "HEX";

#[derive(Copy,Clone,PartialEq)]
enum Section { Global, Text, Data }

#[derive(Debug,Copy,Clone,PartialEq)]
enum Token {
    // Registers
    Zero, AT, V0, V1, A0, A1, A2, A3, T0, T1, T2,
    T3, T4, T5, T6, T7, S0, S1, S2, S3, S4,
    S5, S6, S7, T8, T9, K0, K1, GP, SP, FP, RA,
    // Instructions
    SW, LW, ADDI, BEQ, BNE, AND, OR, ADD, SUB, SLT,
    SRL, SLL, JR, J, JAL,
    // Immediate values
    I,
    // Directives
    Global, Text, Data, Word, Space,
    // Jump and Data Labels
    Label,
    // Can't find
    NotFound,
}

fn main(){
    let args : Vec<String> = std::env::args().collect();

    if args.len() <= 1 {
        eprintln!("ERROR: Can't execute without arguments!");
        std::process::exit(1);
    }
    else if args.len() >= 3 {
        eprintln!("ERROR: Too many arguments!");
        std::process::exit(1);
    }

    let arg1_ext = args[1][args[1].len()-4..args[1].len()].to_lowercase();

    if arg1_ext != ".asm" {
        eprintln!("ERROR: Use an .asm mips assembly file as an argument!");
        std::process::exit(1);
    }

    let fdata = std::fs::read_to_string(&args[1]).expect("ERROR: Can't read file or file doesn't exist!");

    let mut fvec : Vec<&str> = fdata.split("\n").collect();
    fvec.retain(|&x| x.len() > 3);

    // Begin program.mif
    println!("-- {}", TOP_PROGRAM_COMMENT);
    println!("Depth = {};", DEPTH);
    println!("Width = {};", WIDTH);
    println!("Address_radix = {};", ADDRESS_RADIX);
    println!("Data_radix = {};", DATA_RADIX);
    println!("Content");
    println!("Begin");

    // .globl means outside either .text or .data, .text means the program instructions/code, .data means program data
    let mut section : Section = Section::Global;
    let mut word_count : u32 = 0; // Used for jump labels
    let mut mem_byte_alignment : u32 = 0; // Used for data labels
    let mut jump_labels : HashMap<String, u32> = HashMap::new();
    let mut data_labels : HashMap<String, u32> = HashMap::new();
    // Get all Jump and Data labels
    for (i, line) in fvec.iter().enumerate() {
        let no_com_line : &str = &line[..line.find('#').unwrap_or(line.len())]; // Strip comments
        if no_com_line.trim().is_empty() { continue; } // Skip empty lines
        let mut no_com_line_it = no_com_line.trim().split_whitespace();
        let str_tok : String = String::from(no_com_line_it.next().unwrap());
        let enum_tok = parse_first_token(&str_tok);
        match enum_tok {
            Token::Text => {
                section = Section::Text;
                word_count = 0;
            }
            Token::Data => {
                section = Section::Data;
                word_count = 0;
                mem_byte_alignment = 0;
            }
            Token::Label => match section {
                Section::Global => panic!("ERROR({}): There shouldn't be any free labels at the global section!", i+1),
                Section::Text => {
                    let label : &str = &str_tok[..str_tok.len()-1];
                    if data_labels.contains_key(label) {
                        panic!("ERROR({}): The label {} was defined twice or more times!", i+1, label);
                    }
                    jump_labels.insert(String::from(label), word_count);
                }
                Section::Data => {
                    let data_sz : u32 = match no_com_line_it.next().unwrap() {
                        ".word" | ".space" => 4,
                        ".byte" => panic!("ERROR({}): Use .word instead of .byte because of memory alignment.", i+1),
                        ".float" | ".double" => panic!("ERROR({}): There are no float-altering instructions implemented, so you can't use .float", i+1),
                        _ => panic!("ERROR({}): {} is not implemented!", i+1, str_tok),
                    };
                    let label : &str = &str_tok[..str_tok.len()-1];
                    if jump_labels.contains_key(label) { panic!("ERROR({}): The label {} was defined twice or more times!", i+1, label); }
                    // Using byte memory alignment here instead of word alignment for maybe future use
                    data_labels.insert(String::from(label), mem_byte_alignment);
                    for _ in no_com_line_it { mem_byte_alignment += data_sz; }
                }
            }
            _ => (),
        }
        match section {
            // NOTE: This leaves a lot to be desired, and requires strict syntax enforcing
            Section::Text => if enum_tok != Token::Text && enum_tok != Token::Label { word_count += 1; },
            Section::Data => if enum_tok != Token::Data { word_count += 1; },
            _ => (),
        }
        //println!("{{(Str_Token)={} (Enum_Token)={:?}}} ", str_tok, enum_tok);
    }
    // NOTE: This is done because I'm lazy
    if jump_labels["main"] != 0 {
        eprintln!("ERROR: The main label should always be the starting one before any code.");
        std::process::exit(1);
    }

    // Generate each instruction
    for (i, line) in fvec.iter().enumerate() {
        let no_com_line : &str = &line[..line.find('#').unwrap_or(line.len())]; // Strip comments
        if no_com_line.trim().is_empty() { continue; } // Skip empty lines
        let mut args : HashMap<String, Option<u32>> = HashMap::new(); // This will hold the instruction's arguments
        let mut no_com_line_it = no_com_line.trim().split_whitespace(); // no comments line iterator
        let first_str_tok : String = String::from(no_com_line_it.next().unwrap());
        let first_enum_tok : Token = parse_first_token(&first_str_tok);
        if first_enum_tok == Token::Text {
            section = Section::Text;
            word_count = 0;
            continue;
        }
        else if section == Section::Text && first_enum_tok != Token::Label { word_count += 1; }
        else if section != Section::Text { continue; }
        for word in no_com_line_it {
            let str_tok : String = String::from(word.replace(&[','][..], ""));
            let enum_tok = parse_token(&str_tok);
            match first_enum_tok {
                //// Instructions
                //SW, LW, ADDI, BEQ, BNE, AND, OR, ADD, SUB, SLT,
                //SRL, SLL, JR, J, JAL,

                // I-Format Instructions
                Token::SW | Token::LW | Token::ADDI | Token::BEQ | Token::BNE => {
                    args.insert(String::from("opcode"), Some(
                            match first_enum_tok {
                                Token::SW => 0x2B,
                                Token::LW => 0x23,
                                Token::ADDI => 0x08,
                                Token::BEQ => 0x04,
                                Token::BNE => 0x05,
                                // TODO: check if this condition ever reaches
                                _ => panic!("ERROR({}): Instruction {} is not implemented.", i+1, first_str_tok),
                            }
                            << 26));
                    if (enum_tok as u32) < 32 {
                        if args.contains_key("rt") { args.insert(String::from("rs"), Some((enum_tok as u32) << 21)); }
                        else { args.insert(String::from("rt"), Some((enum_tok as u32) << 16)); }
                    }
                    else if enum_tok == Token::I {
                        if str_tok.starts_with("0x") && u32::from_str_radix(&str_tok[2..], 16).is_ok() {
                            args.insert(String::from("i"), Some(u32::from_str_radix(&str_tok[2..], 16).unwrap()));
                        }
                        else {
                            args.insert(String::from("i"), Some(str_tok.parse::<u32>().unwrap()));
                        }
                    }
                    else if enum_tok == Token::NotFound {
                        if data_labels.contains_key(&str_tok) {
                            args.insert(String::from("i"), Some(data_labels[&str_tok] >> 2)); // Divide by 4 since it's word aligned
                        }
                        else if jump_labels.contains_key(&str_tok) {
                            args.insert(String::from("i"), Some((jump_labels[&str_tok] as i32 - word_count as i32) as u32 & 0xFFFF));
                        }
                        else if str_tok.contains("(") && str_tok.contains(")"){
                            let offset : u32 =
                                if str_tok.starts_with("0x") && u32::from_str_radix(&str_tok[2..str_tok.find('(').unwrap()], 16).is_ok() {
                                    u32::from_str_radix(&str_tok[2..str_tok.find('(').unwrap()], 16).unwrap()
                                } else { str_tok[..str_tok.find('(').unwrap()].parse::<u32>().unwrap() };
                            let label : &str = &str_tok[str_tok.find('(').unwrap() + 1..str_tok.find(')').unwrap()];
                            if !data_labels.contains_key(&String::from(label)) {
                                panic!("ERROR({}): Label {} does not exist!", i+1, label);
                            }
                            args.insert(String::from("i"), Some((data_labels[&String::from(label)] >> 2) + offset));
                        }
                        else { panic!("ERROR({}): Label {} does not exist!", i+1, str_tok); }
                    }
                    else { panic!("ERROR({}): {} instruction has wrong syntax! This is wrong: {}", i+1,
                        match first_enum_tok {
                            Token::SW => "SW",
                            Token::LW => "LW",
                            Token::I => "I",
                            Token::BEQ => "BEQ",
                            Token::BNE => "BNE",
                            // TODO: check if this condition ever reaches
                            _ => "",
                        }, str_tok); }
                }

                // R-Format Instructions
                Token::AND | Token::OR | Token::ADD | Token::SUB | Token::SLT | Token::SRL | Token::SLL | Token::JR => {
                    args.insert(String::from("func"), Some(
                            match first_enum_tok {
                                Token::AND => 0x24,
                                Token::OR => 0x25,
                                Token::ADD => 0x20,
                                Token::SUB => 0x22,
                                Token::SLT => 0x2A,
                                Token::SRL => 0x02,
                                Token::SLL => 0x00,
                                Token::JR => 0x08,
                                // TODO: check if this condition ever reaches
                                _ => panic!("ERROR({}): Instruction {} is not implemented.", i+1, first_str_tok),
                            }));
                    if (enum_tok as u32) < 32 {
                        if args.contains_key("rs") ||
                            (args.contains_key("rd") && (first_enum_tok == Token::SLL || first_enum_tok == Token::SRL)) {
                            args.insert(String::from("rt"), Some((enum_tok as u32) << 16));
                        }
                        else if args.contains_key("rd") || first_enum_tok == Token::JR {
                            args.insert(String::from("rs"), Some((enum_tok as u32) << 21));
                        }
                        else { args.insert(String::from("rd"), Some((enum_tok as u32) << 11)); }
                    }
                    else if enum_tok == Token::I {
                        if str_tok.starts_with("0x") && u32::from_str_radix(&str_tok[2..], 16).is_ok() {
                            args.insert(String::from("shamt"), Some(u32::from_str_radix(&str_tok[2..], 16).unwrap()));
                        }
                        else {
                            args.insert(String::from("shamt"), Some(str_tok.parse::<u32>().unwrap() << 6));
                        }
                    }
                    else { panic!("ERROR({}): {} instruction has wrong syntax! This is wrong: {}", i+1,
                        match first_enum_tok {
                            Token::AND => "AND",
                            Token::OR => "OR",
                            Token::ADD => "ADD",
                            Token::SUB => "SUB",
                            Token::SLT => "SLT",
                            Token::SRL => "SRL",
                            Token::SLL => "SLL",
                            Token::JR => "JR",
                            _ => "",
                        }, str_tok); }
                }

                // J-Format instructions
                Token::J | Token::JAL => {
                    args.insert(String::from("opcode"), Some(
                            match first_enum_tok {
                                Token::J => 0x02,
                                Token::JAL => 0x03,
                                // TODO: check if this condition ever reaches
                                _ => panic!("ERROR({}): Instruction {} is not implemented.", i+1, first_str_tok),
                            }
                            << 26));
                    if enum_tok == Token::NotFound {
                        if jump_labels.contains_key(&str_tok) {
                            args.insert(String::from("i"), Some(jump_labels[&str_tok] & 0x3FFFFFF));
                        }
                        else { panic!("ERROR({}): Label {} does not exist!", i+1, str_tok); }
                    }
                    else { panic!("ERROR({}): {} instruction has wrong syntax! This is wrong: {}", i+1,
                        match first_enum_tok {
                            Token::J => "J",
                            Token::JAL => "JAL",
                            // TODO: check if this condition ever reaches
                            _ => "",
                        }, str_tok); }
                }
                _ => { }
            }
        }
        // Here we have all the tokens data and can finally make the instruction in hexadecimal
        if !args.is_empty() {
            //println!("Instruct: {:?} Args: {:?}", first_enum_tok, args);
            println!(
                "\t{:02X}: {:08X};", word_count - 1,
                args.remove("opcode").unwrap_or(Some(0)).unwrap() |
                args.remove("addr")  .unwrap_or(Some(0)).unwrap() | // J - Format only
                args.remove("rs")    .unwrap_or(Some(0)).unwrap() |
                args.remove("rt")    .unwrap_or(Some(0)).unwrap() |
                args.remove("i")     .unwrap_or(Some(0)).unwrap() | // I - Format only
                args.remove("rd")    .unwrap_or(Some(0)).unwrap() | // R - Format only
                args.remove("shamt") .unwrap_or(Some(0)).unwrap() | // R - Format only
                args.remove("func")  .unwrap_or(Some(0)).unwrap()   // R - Format only
                )
        }
    }
    if word_count - 1 < 0xFF { println!("\t[{:02X}..FF]: 00000000;", word_count - 1); }
    else if word_count > DEPTH {
        eprintln!("ERROR: Too many instructions! There can be at most {} instructions!", DEPTH);
        std::process::exit(1);
    }

    // End program.mif
    println!("End;");

    // Begin dmemory.mif
    println!("-- {}", TOP_DATA_COMMENT);
    println!("Depth = {};", DEPTH);
    println!("Width = {};", WIDTH);
    println!("Address_radix = {};", ADDRESS_RADIX);
    println!("Data_radix = {};", DATA_RADIX);
    println!("Content");
    println!("Begin");

    for (i, line) in fvec.iter().enumerate() {
        let no_com_line : &str = &line[..line.find('#').unwrap_or(line.len())]; // Strip comments
        if no_com_line.trim().is_empty() { continue; } // Skip empty lines
        let mut no_com_line_it = no_com_line.trim().split_whitespace(); // no comments line iterator
        let first_str_tok : String = String::from(no_com_line_it.next().unwrap());
        let first_enum_tok : Token = parse_first_token(&first_str_tok);
        if first_enum_tok == Token::Data {
            section = Section::Data;
            word_count = 0;
            continue;
        }
        else if section == Section::Data && first_enum_tok != Token::Label {
            panic!("ERROR({}): Make sure data labels are inline with the data itself!", i+1);
        }
        else if section != Section::Data { continue; }
        let second_str_tok : String = String::from(no_com_line_it.next().unwrap());
        let second_enum_tok : Token = parse_token(&second_str_tok);
        match second_enum_tok {
            Token::Word | Token::Space => (),
            _ => panic!("ERROR({}): Unsupported directive {}! Only .word and .space are implemented.", i+1, second_str_tok),
        }
        if second_enum_tok == Token::Space {
            println!("\t{:02X} : 00000000;", word_count);
            word_count += 1;
            continue;
        }
        for word in no_com_line_it {
            print!("\t{:02X} : ", word_count);
            word_count += 1;
            let val_str_tok : String = String::from(word.replace(&[','][..], ""));
            let val_enum_tok : Token = parse_token(&val_str_tok);
            if val_enum_tok != Token::I {
                panic!("ERROR({}): Unsupported token {}! Only immediate values can come after .word or .space", i+1, val_str_tok);
            }
            if val_str_tok.starts_with("0x") && u32::from_str_radix(&val_str_tok[2..], 16).is_ok() {
                println!("{:08X};", u32::from_str_radix(&val_str_tok[2..], 16).unwrap());
            }
            else { println!("{:08X};", val_str_tok.parse::<u32>().unwrap()); }
        }
    }
    if word_count < 0xFF { println!("\t[{:02X}..FF] : 00000000;", word_count); }
    else if word_count > DEPTH {
        eprintln!("ERROR: Too much data! There can be at most {} words of data!", DEPTH);
        std::process::exit(1);
    }

    // End dmemory.mif
    println!("End;");

    // Debug information
    //println!("{:?}", jump_labels);
    //println!("{:?}", data_labels);
}

fn parse_first_token(s : &String) -> Token {
    // Check if it is an directive
    if s.starts_with(".") {
        match &s[1..] {
            "globl" => Token::Global,
            "text" => Token::Text,
            "data" => Token::Data,
            "word" => Token::Word,
            _ => Token::NotFound,
        }
    }
    // Check if it is an label
    else if s.chars().last().unwrap_or(' ') == ':'{
        Token::Label
    }
    else {
        match s.as_str() {
            "sw" => Token::SW,
            "lw" => Token::LW,
            "addi" => Token::ADDI,
            "beq" => Token::BEQ,
            "bne" => Token::BNE,
            "and" => Token::AND,
            "or" => Token::OR,
            "add" => Token::ADD,
            "sub" => Token::SUB,
            "slt" => Token::SLT,
            "srl" => Token::SRL,
            "sll" => Token::SLL,
            "jr" => Token::JR,
            "j" => Token::J,
            "jal" => Token::JAL,
            _ => Token::NotFound,
        }
    }
}

fn parse_token(z : &String) -> Token {
    let is_immediate = z.parse::<i16>();
    let mut tok = match is_immediate {
        Ok(_) => Token::I,
        Err(_) => Token::NotFound,
    };
    if tok == Token::NotFound {
        // Check if it is an immediate value in hexadecimal
        if z.starts_with("0x") && u32::from_str_radix(&z[2..], 16).is_ok() {
            tok = Token::I;
        }
        // Check if it is an directive
        else if z.starts_with(".") {
            tok = match &z[1..] {
                "globl" => Token::Global,
                "text" => Token::Text,
                "data" => Token::Data,
                "word" => Token::Word,
                "space" => Token::Space,
                _ => Token::NotFound,
            }
        }
        // Check if it is an label
        else if z.chars().last().unwrap_or(' ') == ':'{
            tok = Token::Label;
        }
        //else { tok = parse_token(&z); }
        else {
            tok = match z.as_str() {
                "$zero" => Token::Zero,
                "$at" => Token::AT,
                "$v0" => Token::V0,
                "$v1" => Token::V1,
                "$a0" => Token::A0,
                "$a1" => Token::A1,
                "$a2" => Token::A2,
                "$a3" => Token::A3,
                "$t0" => Token::T0,
                "$t1" => Token::T1,
                "$t2" => Token::T2,
                "$t3" => Token::T3,
                "$t4" => Token::T4,
                "$t5" => Token::T5,
                "$t6" => Token::T6,
                "$t7" => Token::T7,
                "$s0" => Token::S0,
                "$s1" => Token::S1,
                "$s2" => Token::S2,
                "$s3" => Token::S3,
                "$s4" => Token::S4,
                "$s5" => Token::S5,
                "$s6" => Token::S6,
                "$s7" => Token::S7,
                "$t8" => Token::T8,
                "$t9" => Token::T9,
                "$k0" => Token::K0,
                "$k1" => Token::K1,
                "$gp" => Token::GP,
                "$sp" => Token::SP,
                "$fp" => Token::FP,
                "$ra" => Token::RA,
                "sw" => Token::SW,
                "lw" => Token::LW,
                "addi" => Token::ADDI,
                "beq" => Token::BEQ,
                "bne" => Token::BNE,
                "and" => Token::AND,
                "or" => Token::OR,
                "add" => Token::ADD,
                "sub" => Token::SUB,
                "slt" => Token::SLT,
                "srl" => Token::SRL,
                "sll" => Token::SLL,
                "jr" => Token::JR,
                "j" => Token::J,
                "jal" => Token::JAL,
                _ => Token::NotFound,
            };
        }
    }
    tok
}
