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

use embedded_graphics::{
    prelude::*,
    Drawing,
    drawable::Pixel,
    unsignedcoord::{UnsignedCoord},
    primitives::Rect,
    fonts::{
        Font6x8, Font12x16
    },
};
use alloc::{
    vec::Vec,
    boxed::Box
};
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout as AllocLayout;
use core::panic::PanicInfo;
use rt::{entry, exception};
use stm32f7::stm32f7x6::{
    CorePeripherals,
    Peripherals,
    I2C3,
};
use stm32f7_discovery::{
    gpio::{GpioPort, OutputPin},
    init,
    lcd::{self, Color, HEIGHT, WIDTH, Framebuffer},
    random::Rng,
    system_clock::{self, Hz},
    touch,
    i2c::I2C,
};


use geometry::{AABBox, Point};
use player::{Player, Collide, CollideSelf, PAD_LEFT, PAD_RIGHT};

use buffs::{
    Buff, ChangeDirBuff, ClearBuff, FastPlayerBuffSprite, SlowBuff, ColorBuff,
    BigBuff, SmallBuff};
use display::{GameColor, LcdDisplay};
use embedded_graphics::coord::Coord;

const HEAP_SIZE: usize = 1024 * 1024; // in bytes

const C_PLAYER_A: GameColor = GameColor{value: 0x00_00FF};
const C_PLAYER_B: GameColor = GameColor{value: 0x00_FF00};

const TOP_LEFT: Point = Point { x: 0, y: 0 };
const TOP_MID: Point = Point { x: WIDTH / 2, y: 0 };
const MID_MID: Point = Point {
    x: WIDTH / 2,
    y: HEIGHT / 2,
};
const BOTTOM_MID: Point = Point {
    x: WIDTH / 2,
    y: HEIGHT,
};
const BOTTOM_RIGHT: Point = Point {
    x: WIDTH,
    y: HEIGHT,
};
const LEFT_MID: Point = Point {
    x: 0,
    y: HEIGHT / 2,
};
const RIGHT_MID: Point = Point {
    x: WIDTH,
    y: HEIGHT / 2,
};

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

    let cooldown: i32 = 3 * 100;

    let mut cool_ticks = system_clock::ticks();
    loop {
        let ticks = system_clock::ticks();
        let d_ticks = ticks - cool_ticks;

        if (d_ticks as i32) < cooldown {
            display.draw(Font12x16::render_str(&format!("BE READY! FUN STARS IN {} SECONDS!", (cooldown - d_ticks as i32) / 100))
                            .with_stroke(Some(C_PLAYER_A))
                            .with_fill(Some(GameColor {value: 0x00_0000}))
                            .translate(Coord::new((WIDTH/2) as i32 - 200, (HEIGHT/2) as i32))
                            .into_iter());
        } else {
            display.clear();
            play_game(&mut rng, &mut display, &mut i2c_3);
            cool_ticks = system_clock::ticks();
        }
    }
}

fn play_game<F: Framebuffer>(rng: &mut Rng, display: &mut LcdDisplay<F>, i2c_3: &mut I2C<I2C3>) {
    let y_offset: u32 = 65;
    let mut score_b: u32 = 0;
    let mut score_a: u32 = 0;
    display.draw(Font6x8::render_str(&format!("<--- Player B: {:04}  --->", score_b))
                            .with_stroke(Some(C_PLAYER_B))
                            .with_fill(Some(GameColor {value: 0x00_0000}))
                            .translate(Coord::new(y_offset as i32, 0))
                            .into_iter()
                            .map(|p| Pixel(UnsignedCoord::new((10 - p.0[1])as u32, p.0[0]), p.1)));

    display.draw(Font6x8::render_str(&format!("<--- Player A: {:04}  --->", score_a))
                            .with_stroke(Some(C_PLAYER_A))
                            .with_fill(Some(GameColor {value: 0x00_0000}))
                            .translate(Coord::new(y_offset as i32, 0))
                            .into_iter()
                            .map(|p| Pixel(UnsignedCoord::new((WIDTH as u32 - 10 + p.0[1])as u32, HEIGHT as u32 - p.0[0]), p.1)));
    // Start game loop
    match game_loop(rng, display, i2c_3) {
        0 => score_a += 1,
        1 => score_b += 1,
        _ => {},
    }
}

fn game_loop<F>(rng: &mut Rng, display: &mut LcdDisplay<F>, i2c_3: &mut I2C<I2C3>) -> u32
where F: Framebuffer {

    let pos_a = (
        PAD_LEFT + get_rand_num(rng) as f32 % (WIDTH as f32 - PAD_LEFT - PAD_RIGHT),
        get_rand_num(rng) as f32 % HEIGHT as f32,
    );
    let pos_b = (
        PAD_LEFT + get_rand_num(rng) as f32 % (WIDTH as f32 - PAD_LEFT - PAD_RIGHT),
        get_rand_num(rng) as f32 % HEIGHT as f32,
    );
    let angle_a = get_rand_num(rng) as f32 % 360_f32;
    let angle_b = get_rand_num(rng) as f32 % 360_f32;

    let player_a = Player::new(
        AABBox::new(MID_MID, BOTTOM_RIGHT),
        AABBox::new(TOP_MID, RIGHT_MID),
        C_PLAYER_A,
        pos_a,
        2,
        angle_a,
    );
    let player_b = Player::new(
        AABBox::new(TOP_LEFT, MID_MID),
        AABBox::new(LEFT_MID, BOTTOM_MID),
        C_PLAYER_B,
        pos_b,
        2,
        angle_b,
    );
    let mut buffs: Vec<Box<Buff>> = Vec::new();
    let mut players: Vec<Player> = Vec::new();
    players.push(player_a);
    players.push(player_b);

    let mut last_curve_update = system_clock::ticks();
    let mut last_time_update = system_clock::ticks();
    let mut next_buff = get_rand_num(rng) % 100;
    let mut last_buff = system_clock::ticks();
    let mut player_collision = false;
    loop {
        let mut touches: Vec<Point> = Vec::new();
        for touch in &touch::touches(i2c_3).unwrap() {
            touches.push(Point {
                x: touch.x as usize,
                y: touch.y as usize,
            });
        }

        let ticks = system_clock::ticks();

        if ticks - last_time_update > 10 {
            display.draw(Font6x8::render_str(&format!("Time {:04}", ticks / 10))
                            .with_stroke(Some(C_PLAYER_A))
                            .with_fill(Some(GameColor {value: 0x00_0000}))
                            .into_iter());
            last_time_update = ticks;
        }
        if ticks - last_buff >= next_buff as usize {
            next_buff = get_rand_num(rng) % (100 * 30);
            last_buff = system_clock::ticks();
            buffs.push(new_rand_buff(rng));
        }
        if !player_collision && ticks - last_curve_update >= 3 {
            for p in &mut players {
                p.act(&touches);
            }

            last_curve_update = ticks;

            // player player collision
            for i in 1..=players.len() {
                let (pis, pjs) = players.split_at(i);
                let pi = pis.last().unwrap();

                if pi.collides() {
                    player_collision = true;
                    if cfg!(debug_assertions) {println!("self collision");}
                } else  { 
                    for pj in pjs {
                        if pi.collides_with(pj) {
                            player_collision = true;
                            if cfg!(debug_assertions) {println!("collision A");}
                        } else if pj.collides_with(pi) {
                            player_collision = true;
                            if cfg!(debug_assertions) {println!("collision B");}
                        }
                    }
                }
            }

            // player buff collision
            let mut clear_all = false;
            for p in &mut players {
                let mut i: usize = 0;
                while i < buffs.len() {
                    if p.collides_with(&buffs[i]) {
                        buffs[i].apply_player(p);
                        clear_all |= buffs[i].clear_screen();
                        let aabb = buffs[i].aabb();
                        display.draw(Rect::new(aabb.0, aabb.1)
                                        .with_fill(Some(GameColor{value: 0x00_0000}))
                                        .into_iter());
                        buffs.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
            if clear_all {
                display.clear();
                // TODO: clear players
            }
            for p in &mut buffs {
                display.draw(p.draw());
            }
            for p in &mut players {
                p.draw(display);
            }
        }
    }
}

fn new_rand_buff(rng: &mut Rng) -> Box<Buff + 'static> {
    let pos_buff = (
        (PAD_LEFT + get_rand_num(rng) as f32 
            % (WIDTH as f32 - PAD_LEFT - PAD_RIGHT)) as i32,
        (get_rand_num(rng) as f32 % HEIGHT as f32) as i32,
    );
    let rand = get_rand_num(rng);
    match (rand % 7) +3 {
        0 => Box::new(FastPlayerBuffSprite::new(Coord::new(pos_buff.0, pos_buff.1))),
        1 => Box::new(ClearBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        2 => Box::new(ChangeDirBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        3 => Box::new(SlowBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        4 => Box::new(ColorBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        5 => Box::new(BigBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        6 => Box::new(SmallBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        _ => Box::new(SlowBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
    }
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
