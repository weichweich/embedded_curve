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
#[macro_use]
extern crate stm32f7;
#[macro_use]
extern crate stm32f7_discovery;

mod curve;
pub mod geometry;
pub mod input;
pub mod display;
pub mod draw;

use libm;
use alloc::vec::Vec;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout as AllocLayout;
use core::panic::PanicInfo;
use rt::{entry, exception};
use stm32f7::stm32f7x6::{CorePeripherals, Peripherals};
use stm32f7_discovery::{
    gpio::{GpioPort, OutputPin},
    init,
    system_clock::{self, Hz},
    lcd::{self, Color, WIDTH, HEIGHT},
    touch,
};
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;
use embedded_graphics::coord::Coord;
use embedded_graphics::primitives::{Circle, Rect};
use curve::{
    Curve, CurveField
};

use draw::{
    draw_line,
    draw_triangle
};

use geometry::{
    Point, AABBox, Vector2D
};
use input::{
    Player, PlayerInput,
};
use display::{LcdDisplay, GameColor};

const HEAP_SIZE: usize = 50 * 1024; // in bytes

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

    println!("Hello World");

    let mut i2c_3 = init::init_i2c_3(peripherals.I2C3, &mut rcc);
    i2c_3.test_1();
    i2c_3.test_2();
    // touch initialization should be done after audio initialization, because the touch
    // controller might not be ready yet
    touch::check_family_id(&mut i2c_3).unwrap();

    let mut display = LcdDisplay::new(&mut layer_1);

    let top_left = Point { x: 0, y: 0};
    let top_mid = Point { x: WIDTH/ 2, y: 0};
    let bottom_left = Point { x: 0, y: HEIGHT};
    let mid_mid = Point { x: WIDTH / 2, y: HEIGHT / 2};
    let bottom_mid = Point { x: WIDTH / 2, y: HEIGHT};
    let bottom_right = Point { x: WIDTH, y: HEIGHT};
    let left_mid = Point { x: 0, y: HEIGHT / 2 };
    let right_mid = Point { x: WIDTH, y: HEIGHT / 2 };

    let mut player_a = Player::new(
        AABBox::new(top_left, mid_mid), 
        AABBox::new(left_mid, bottom_mid),
        GameColor{value:0x0000FF}, (100.0,  130.0),
        1, Vector2D{x: 1.0, y: 1.0});
    let mut player_b = Player::new(
        AABBox::new(mid_mid, bottom_right),
        AABBox::new(top_mid, right_mid), 
        GameColor{value: 0x00FF00}, (380.0,  130.0), 
        1, Vector2D{x: -1.0, y: -1.0});

    let mut last_curve_update = system_clock::ticks();
    // let mut opt_last_point = None;

    loop {
        // poll for new touch data
        let mut touches: Vec<Point> = Vec::new();
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            touches.push(Point{x: touch.x as usize, y: touch.y as usize});
        }

        player_b.act(&touches);
        player_a.act(&touches);
        player_a.draw(&mut display);
        player_b.draw(&mut display);

        last_curve_update = system_clock::ticks();
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
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    use cortex_m::asm;
    use cortex_m_semihosting::hio;

    if let Ok(mut hstdout) = hio::hstdout() {
        let _ = writeln!(hstdout, "{}", info);
    }

    // OK to fire a breakpoint here because we know the microcontroller is connected to a debugger
    asm::bkpt();

    loop {}
}
