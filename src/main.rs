mod cpu;

use cpu::CPU;


fn main() {
    println!("Hello, world!");
    let program: Vec<u8> = vec![0xa9, 0x05, 0x00];

    let mut cpu = CPU::new();
    println!("content in reg a is {}", cpu.register_a);
    cpu.interpret(program); 
    println!("content in reg a is {}", cpu.register_a);
}
