#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(alloc)]
#![feature(lang_items)]

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
pub mod draw;
pub mod geometry;
pub mod player;
pub mod playingfield;

use embedded_graphics::Drawing;
use alloc::{
    vec::Vec,
    boxed::Box
};
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout as AllocLayout;
use core::panic::PanicInfo;
use rt::{entry, exception};
use stm32f7::stm32f7x6::{CorePeripherals, Peripherals};
use stm32f7_discovery::{
    gpio::{GpioPort, OutputPin},
    init,
    lcd::{self, Color, HEIGHT, WIDTH},
    random::Rng,
    system_clock::{self, Hz},
    touch,
};

use geometry::{AABBox, Point};
use player::Player;

use buffs::{Buff, ChangeDirBuff, ClearBuff, FastPlayerBuffSprite, SlowBuff};
use display::{GameColor, LcdDisplay};
use embedded_graphics::coord::Coord;
use playingfield::PlayingField;

const HEAP_SIZE: usize = 1024 * 1024; // in bytes

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

    lcd.set_background_color(Color::from_hex(0x001000));
    let mut layer_1 = lcd.layer_1().unwrap();
    let mut layer_2 = lcd.layer_2().unwrap();

    layer_2.clear();
    layer_1.clear();

    // Make `println` print to the LCD
    lcd::init_stdout(layer_2);
    if cfg!(debug_assertions) {
        println!("Start Game");
    }

    println!("{}", HEAP_SIZE);

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

    let top_left = Point { x: 0, y: 0 };
    let top_mid = Point { x: WIDTH / 2, y: 0 };
    let mid_mid = Point {
        x: WIDTH / 2,
        y: HEIGHT / 2,
    };
    let bottom_mid = Point {
        x: WIDTH / 2,
        y: HEIGHT,
    };
    let bottom_right = Point {
        x: WIDTH,
        y: HEIGHT,
    };
    let left_mid = Point {
        x: 0,
        y: HEIGHT / 2,
    };
    let right_mid = Point {
        x: WIDTH,
        y: HEIGHT / 2,
    };

    let pos_a = (
        get_rand_num(&mut rng) as f32 % WIDTH as f32,
        get_rand_num(&mut rng) as f32 % HEIGHT as f32,
    );
    let pos_b = (
        get_rand_num(&mut rng) as f32 % WIDTH as f32,
        get_rand_num(&mut rng) as f32 % HEIGHT as f32,
    );
    let angle_a = get_rand_num(&mut rng) as f32 % 360_f32;
    let angle_b = get_rand_num(&mut rng) as f32 % 360_f32;

    //ID for Objects 0 = default and 1..255 for objects!!!
    let player_a = Player::new(
        AABBox::new(left_mid, bottom_mid),
        AABBox::new(top_left, mid_mid),
        GameColor { value: 0x0000FF },
        pos_a,
        2,
        angle_a,
        1,
    );
    let player_b = Player::new(
        AABBox::new(top_mid, right_mid),
        AABBox::new(mid_mid, bottom_right),
        GameColor { value: 0x00FF00 },
        pos_b,
        2,
        angle_b,
        2,
    );
    let mut players: Vec<Player> = Vec::new();
    players.push(player_a);
    players.push(player_b);

    let mut buffs: Vec<Box<Buff>> = Vec::new();
    let pos_buff = (
        get_rand_num(&mut rng) as f32 % WIDTH as f32,
        get_rand_num(&mut rng) as f32 % HEIGHT as f32,
    );
    buffs.push(Box::new(
        FastPlayerBuffSprite::new(Coord::new(pos_buff.0 as i32, pos_buff.1 as i32), 3)));
    let pos_buff = (
        get_rand_num(&mut rng) as f32 % WIDTH as f32,
        get_rand_num(&mut rng) as f32 % HEIGHT as f32,
    );
    buffs.push(Box::new(
        ClearBuff::new(Coord::new(pos_buff.0 as i32, pos_buff.1 as i32), 4)));
    let pos_buff = (
        get_rand_num(&mut rng) as f32 % WIDTH as f32,
        get_rand_num(&mut rng) as f32 % HEIGHT as f32,
    );
    buffs.push(Box::new(
        ChangeDirBuff::new(Coord::new(pos_buff.0 as i32, pos_buff.1 as i32), 5)));
    let pos_buff = (
        get_rand_num(&mut rng) as f32 % WIDTH as f32,
        get_rand_num(&mut rng) as f32 % HEIGHT as f32,
    );
    buffs.push(Box::new(
        SlowBuff::new(Coord::new(pos_buff.0 as i32, pos_buff.1 as i32), 6)));

    let mut last_curve_update = system_clock::ticks();
    // let mut opt_last_point = None;
    let mut playingfield = PlayingField::new();
    let player_thing = system_clock::ticks();
    let mut thing = 0;

    loop {
        // poll for new touch data
        let mut touches: Vec<Point> = Vec::new();
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            touches.push(Point {
                x: touch.x as usize,
                y: touch.y as usize,
            });
        }
        // println!("hoolahoop");
        let ticks = system_clock::ticks();
        if !playingfield.collision && ticks - last_curve_update >= 3 {
            for p in &mut players {
                p.act(&touches);
                p.draw(&mut display, &mut playingfield);
            }

            for p in &mut buffs {
                display.draw(p.draw());
            }

            last_curve_update = ticks;
        }
        // for c in collisions {
        //     if c.old is player and c.new is player {
        //         old_player.collide_with(new_player)
        //     } else if c.old is player and c.new is buff {
        //         old_player.collide_with(buff)
        //     }
        // }
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
