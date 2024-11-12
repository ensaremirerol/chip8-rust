use clap::Parser;

mod chip;
mod terminal;

#[derive(Parser)]
#[command(version, about, author)]
struct Opts {
    #[arg(short, long, help = "Path to the ROM file")]
    rom: String,
    #[arg(short, long, default_value = "500", help = "Chip8 clock speed in kHz")]
    clock_speed: u64,
    #[arg(short, long, default_value = "false", help = "Enable shift quirk")]
    shift_quirk: bool,
}

fn main() {
    let opts = Opts::parse();
    let rom = std::fs::read(&opts.rom).unwrap_or_else(|e| panic!("Failed to read ROM file: {}", e));
    let mut terminal = terminal::Terminal::new();
    terminal
        .init()
        .unwrap_or_else(|e| panic!("Failed to initialize terminal: {}", e));
    let mut chip = chip::Chip::new(opts.clock_speed, &mut terminal, opts.shift_quirk);
    chip.load_rom(rom);
    loop {
        let ret = chip.run();
        if ret == "exit" {
            break;
        }
    }
    terminal
        .exit()
        .unwrap_or_else(|e| panic!("Failed to exit terminal: {}", e));
}
