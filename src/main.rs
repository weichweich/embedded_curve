#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(alloc)]
#![feature(lang_items)]

#[macro_use]
extern crate alloc;
extern crate alloc_cortex_m;
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting;
extern crate stm32f7;
#[macro_use]
extern crate stm32f7_discovery;

pub mod buffs;
pub mod display;
pub mod geometry;
pub mod player;
pub mod game;
pub mod border;

use stm32f7::stm32f7x6::I2C3;
use stm32f7_discovery::i2c::I2C;
use embedded_graphics::{
    prelude::*,
    Drawing,
    drawable::Pixel,
    unsignedcoord::{UnsignedCoord},
    fonts::{
        Font6x8, Font12x16,
    },
};
use alloc::vec::Vec;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout as AllocLayout;
use core::panic::PanicInfo;
use rt::{entry, exception};
use stm32f7::stm32f7x6::{
    CorePeripherals,
    Peripherals,
};
use stm32f7_discovery::{
    gpio::{GpioPort, OutputPin},
    init,
    lcd::{self, Color, HEIGHT, WIDTH},
    random::Rng,
    system_clock::{self, Hz},
    touch,
};
use embedded_graphics::coord::Coord;


use display::{GameColor, LcdDisplay};
use game::{Game, GameState};

const HEAP_SIZE: usize = 1024 * 1024; // in bytes

const C_PLAYER_A: GameColor = GameColor{value: 0x00_00FF};
const C_PLAYER_B: GameColor = GameColor{value: 0x00_FF00};
const C_PLAYER_C: GameColor = GameColor{value: 0xFF_0000};
const C_PLAYER_D: GameColor = GameColor{value: 0xFF_FF00};
const C_BLACK: GameColor = GameColor{value: 0x00_0000};
const C_WHITE: GameColor = GameColor{value: 0xFF_FFFF};

pub fn to_coord(t: (i32, i32)) -> Coord {
    Coord::new(t.0, t.1)
}

#[entry]
fn main() -> ! {
    let core_peripherals = CorePeripherals::take().unwrap();
    let mut systick = core_peripherals.SYST;

    let peripherals = Peripherals::take().unwrap();
    let mut rcc = peripherals.RCC;
    let mut pwr = peripherals.PWR;
    let mut flash = peripherals.FLASH;
    let mut fmc = peripherals.FMC;
    let mut ltdc = peripherals.LTDC;
    let mut sai_2 = peripherals.SAI2;
    let mut rng = peripherals.RNG;

    init::init_system_clock_216mhz(&mut rcc, &mut pwr, &mut flash);
    init::enable_gpio_ports(&mut rcc);
    init::enable_syscfg(&mut rcc);

    let gpio_a = GpioPort::new(peripherals.GPIOA);
    let gpio_b = GpioPort::new(peripherals.GPIOB);
    let gpio_c = GpioPort::new(peripherals.GPIOC);
    let gpio_d = GpioPort::new(peripherals.GPIOD);
    let gpio_e = GpioPort::new(peripherals.GPIOE);
    let gpio_f = GpioPort::new(peripherals.GPIOF);
    let gpio_g = GpioPort::new(peripherals.GPIOG);
    let gpio_h = GpioPort::new(peripherals.GPIOH);
    let gpio_i = GpioPort::new(peripherals.GPIOI);
    let gpio_j = GpioPort::new(peripherals.GPIOJ);
    let gpio_k = GpioPort::new(peripherals.GPIOK);
    let mut pins = init::pins(
        gpio_a, gpio_b, gpio_c, gpio_d, gpio_e, gpio_f, gpio_g, gpio_h, gpio_i, gpio_j, gpio_k,
    );

    // configure the systick timer 20Hz (20 ticks per second)
    init::init_systick(Hz(100), &mut systick, &rcc);
    systick.enable_interrupt();

    init::init_sdram(&mut rcc, &mut fmc);
    let mut lcd = init::init_lcd(&mut ltdc, &mut rcc);
    pins.display_enable.set(true);
    pins.backlight.set(true);

    // Initialize the allocator BEFORE you use it
    unsafe { ALLOCATOR.init(rt::heap_start() as usize, HEAP_SIZE) }

    lcd.set_background_color(Color::from_hex(0x00_0000));
    let mut layer_1 = lcd.layer_1().unwrap();
    let mut layer_2 = lcd.layer_2().unwrap();

    layer_2.clear();
    layer_1.clear();

    // Make `println` print to the LCD
    lcd::init_stdout(layer_2);
    if cfg!(debug_assertions) {
        println!("Start Game");
        println!("Heap size: {}", HEAP_SIZE);
    }

    let mut i2c_3 = init::init_i2c_3(peripherals.I2C3, &mut rcc);
    i2c_3.test_1();
    i2c_3.test_2();

    init::init_sai_2(&mut sai_2, &mut rcc);
    init::init_wm8994(&mut i2c_3).expect("WM8994 init failed");
    // touch initialization should be done after audio initialization, because the touch
    // controller might not be ready yet
    touch::check_family_id(&mut i2c_3).unwrap();

    let mut rng = Rng::init(&mut rng, &mut rcc).expect("RNG init failed");

    let mut display = LcdDisplay::new(&mut layer_1);

    let num_player = player_select(&mut display, &mut i2c_3);
    let player_c = &[C_PLAYER_A, C_PLAYER_B, C_PLAYER_C, C_PLAYER_D];
    let mut game = Game::new(&player_c[..num_player], &mut rng);
    display.clear();

    loop {
    
        game.new_game(&mut rng);

        ready_screen(&mut display, 3*100);
        display.clear();

        for (i, p) in game.players.iter().enumerate() {
            match i {
                0 => draw_text_right(&mut display, &format!("<--- Player A: {:04}  --->", p.score),
                                C_BLACK, C_PLAYER_A),
                1 => draw_text_left(&mut display, &format!("<--- Player B: {:04}  --->", p.score),
                                C_BLACK, C_PLAYER_B),
                2 => draw_text_top(&mut display, &format!("<--- Player C: {:04}  --->", p.score),
                                C_BLACK, C_PLAYER_C),
                3 => draw_text_bottom(&mut display, &format!("<--- Player D: {:04}  --->", p.score),
                                C_BLACK, C_PLAYER_D),
                _ => {},
            }
        }
        let mut last_ticks = system_clock::ticks();
        let mut touches: Vec<Coord> = Vec::new();
        loop {
            let ticks = system_clock::ticks();
            let d_ticks = ticks - last_ticks;
            if d_ticks < 3 {
                continue;
            }
            last_ticks = system_clock::ticks();

            for touch in &touch::touches(&mut i2c_3).unwrap() {
                touches.push(Coord::new(
                    i32::from(touch.x),
                    i32::from(touch.y),
                ));
            }
            match game.step(&mut rng, &mut display, &touches, d_ticks) {
                GameState::Finished => {
                    let mut msg = "Player ? has won!";
                    for (i, p) in game.players.iter().enumerate() {
                        if !p.lost {
                            match i {
                                0 => msg = "Player A has won!",
                                1 => msg = "Player B has won!",
                                2 => msg = "Player C has won!",
                                3 => msg = "Player D has won!",
                                _ => msg = "Player ? has won!",
                            }
                        }
                    }
                    text_above_mid(&mut display, &msg, C_BLACK, C_WHITE);
                    break;
                },
                GameState::Playing => {},
            }
            touches.clear();
        }
    }
}

fn player_select<D>(display: &mut D, i2c_3: &mut I2C<I2C3>) -> usize
where 
    D: Drawing<GameColor>,
{
    let w1_4 = (WIDTH/4) as i32;
    display.draw(Font12x16::render_str("1")
            .with_stroke(Some(C_PLAYER_A))
            .with_fill(Some(C_BLACK))
            .translate(Coord::new((w1_4 - 12) / 2, (HEIGHT as i32 - 12) / 2))
            .into_iter().chain(
            Font12x16::render_str("2")
            .with_stroke(Some(C_PLAYER_B))
            .with_fill(Some(C_BLACK))
            .translate(Coord::new(w1_4 + (w1_4 - 12) / 2, (HEIGHT as i32 - 12) / 2))
            .into_iter()
            ).chain(
            Font12x16::render_str("3")
            .with_stroke(Some(C_PLAYER_C))
            .with_fill(Some(C_BLACK))
            .translate(Coord::new(w1_4*2 + (w1_4 - 12) / 2, (HEIGHT as i32 - 12) / 2))
            .into_iter()
            ).chain(
            Font12x16::render_str("4")
            .with_stroke(Some(C_PLAYER_D))
            .with_fill(Some(C_BLACK))
            .translate(Coord::new(w1_4*3 + (w1_4 - 12) / 2, (HEIGHT as i32 - 12) / 2))
            .into_iter()
    ));
    loop {
        for touch in &touch::touches(i2c_3).unwrap() {
            if (touch.x as usize) < WIDTH / 4 {
                return 1;
            } else if (touch.x as usize) < WIDTH / 2 {
                return 2;
            } else if (touch.x as usize) < 3* WIDTH / 4 {
                return 3;
            } else {
                return 4;
            } 
        }
    }
}

fn ready_screen<D>(display: &mut D, cooldown: i32)
where
    D: Drawing<GameColor>,
{
    let start_tm = system_clock::ticks();
    let mut passed = (system_clock::ticks() - start_tm) as i32;
    while passed < cooldown {
        huge_text_mid(display, &format!("BE READY! FUN STARTS IN {} SECONDS!!", (cooldown - passed) / 100),
                      C_BLACK, C_PLAYER_A);
        passed = (system_clock::ticks() - start_tm) as i32;
    }
}

fn text_above_mid<'a, D>(display: &mut D, text: &'a str, fill_color: GameColor, 
                         text_color: GameColor)
where 
    D: Drawing<GameColor>,
{
    let len = (text.len() * 12) as i32;
    display.draw(Font12x16::render_str(text)
            .with_stroke(Some(text_color))
            .with_fill(Some(fill_color))
            .translate(Coord::new((WIDTH as i32 - len) / 2, (HEIGHT as i32 - 8) / 2 - 24))
            .into_iter())
}

fn huge_text_mid<'a, D>(display: &mut D, text: &'a str, fill_color: GameColor, 
                        text_color: GameColor)
where 
    D: Drawing<GameColor>,
{
    let len = (text.len() * 12) as i32;
    display.draw(Font12x16::render_str(text)
            .with_stroke(Some(text_color))
            .with_fill(Some(fill_color))
            .translate(Coord::new((WIDTH as i32 - len) / 2, (HEIGHT as i32 - 8) / 2))
            .into_iter())
}

fn draw_text_right<'a, D>(display: &mut D, text: &'a str, fill_color: GameColor,
                        text_color: GameColor)
where D: Drawing<GameColor> {
    let len = (text.len() * 6) as i32;
    display.draw(Font6x8::render_str(text)
                .with_stroke(Some(text_color))
                .with_fill(Some(fill_color))
                .translate(Coord::new((HEIGHT as i32 - len) / 2, 0))
                .into_iter()
                .map(|p| Pixel(UnsignedCoord::new(WIDTH as u32 - 8 + p.0[1],
                                                  HEIGHT as u32 - p.0[0]), p.1)));
}

fn draw_text_left<'a, D>(display: &mut D, text: &'a str, fill_color: GameColor,
                        text_color: GameColor)
where D: Drawing<GameColor> {
    let len = (text.len() * 6 / 2) as i32;
    display.draw(Font6x8::render_str(text)
                .with_stroke(Some(text_color))
                .with_fill(Some(fill_color))
                .translate(Coord::new(len as i32, 0))
                .into_iter()
                .map(|p| Pixel(UnsignedCoord::new((8 - p.0[1]) as u32, p.0[0]), p.1)));
}

fn draw_text_top<'a, D>(display: &mut D, text: &'a str, fill_color: GameColor,
                        text_color: GameColor)
where D: Drawing<GameColor> {
    let len = (text.len() * 6) as i32;
    display.draw(Font6x8::render_str(text)
                .with_stroke(Some(text_color))
                .with_fill(Some(fill_color))
                .translate(Coord::new((WIDTH as i32 - len) / 2, 0))
                .into_iter()
                .map(|p| Pixel(UnsignedCoord::new(WIDTH as u32 - p.0[0],
                                                  8-p.0[1]), p.1)));
}

fn draw_text_bottom<'a, D>(display: &mut D, text: &'a str, fill_color: GameColor,
                           text_color: GameColor)
where D: Drawing<GameColor> {
    let len = (text.len() * 6) as i32;
    display.draw(Font6x8::render_str(text)
                .with_stroke(Some(text_color))
                .with_fill(Some(fill_color))
                .translate(Coord::new((WIDTH as i32 - len) / 2, (HEIGHT - 8) as i32))
                .into_iter());
}

fn get_rand_num(rnd: &mut Rng) -> u32 {
    loop {
        match rnd.poll_and_get() {
            Err(_) => {}
            Ok(num) => {
                break num;
            }
        }
    }
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[exception]
fn SysTick() {
    system_clock::tick();
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn rust_oom(_: AllocLayout) -> ! {
    println!("OOM!!");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    use cortex_m::asm;
    use cortex_m_semihosting::hio;
    println!("PANIC");
    println!("{}", info);
    if let Ok(mut hstdout) = hio::hstdout() {
        let _ = writeln!(hstdout, "{}", info);
    }

    // OK to fire a breakpoint here because we know the microcontroller is connected to a debugger
    asm::bkpt();

    loop {}
}
