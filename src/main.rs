#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc_cortex_m;
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting;
#[macro_use]
extern crate stm32f7;
#[macro_use]
extern crate stm32f7_discovery;

use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout as AllocLayout;
use core::panic::PanicInfo;
use rt::{entry, exception};
use stm32f7::stm32f7x6::{CorePeripherals, Peripherals};
use stm32f7_discovery::{
    gpio::{GpioPort, OutputPin},
    init,
    system_clock::{self, Hz},
    lcd,
    lcd::Color,
    touch,
};

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
    init::init_systick(Hz(20), &mut systick, &rcc);
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

    // // turn led on
    // pins.led.set(true);

    // let mut last_led_toggle = system_clock::ticks();
    // loop {
    //     let ticks = system_clock::ticks();
    //     // every 0.5 seconds (we have 20 ticks per second)
    //     if ticks - last_led_toggle >= 10 {
    //         pins.led.toggle();
    //         last_led_toggle = ticks;
    //     }
    // }


    let mut i2c_3 = init::init_i2c_3(peripherals.I2C3, &mut rcc);
    i2c_3.test_1();
    i2c_3.test_2();
    // touch initialization should be done after audio initialization, because the touch
    // controller might not be ready yet
    touch::check_family_id(&mut i2c_3).unwrap();

    loop {
        // poll for new touch data
        for touch in &touch::touches(&mut i2c_3).unwrap() {
            layer_1.print_point_color_at(
                touch.x as usize,
                touch.y as usize,
                Color::from_hex(0xffff00),
            );
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

// struct TouchTask<S, F>
// where
//     S: Stream<Item = ()>,
//     F: Framebuffer,
// {
//     touch_int_stream: S,
//     i2c_3_mutex: Arc<FutureMutex<I2C<device::I2C3>>>,
//     layer_mutex: Arc<FutureMutex<Layer<F>>>,
// }

// impl<S, F> TouchTask<S, F>
// where
//     S: Stream<Item = ()>,
//     F: Framebuffer,
// {
//     async fn run(self) {
//         let Self {
//             touch_int_stream,
//             i2c_3_mutex,
//             layer_mutex,
//         } = self;
//         pin_mut!(touch_int_stream);
//         await!(layer_mutex.with(|l| l.clear()));
//         loop {
//             await!(touch_int_stream.next()).expect("touch channel closed");
//             let touches = await!(i2c_3_mutex.with(|i2c_3| touch::touches(i2c_3))).unwrap();
//             await!(layer_mutex.with(|layer| for touch in touches {
//                 layer.print_point_color_at(
//                     touch.x as usize,
//                     touch.y as usize,
//                     Color::from_hex(0xffff00),
//                 );
//             }))
//         }
//     }
// }

