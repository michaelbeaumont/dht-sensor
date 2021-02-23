//!  Measure the temperature and humidity from a DHT11 or DHT22 sensor and print
//!  with hprintln (to a gdb session).
//!  The DHT data pin is connected to pin A8 on the MCU board and has 
//!  a pull up resistor. (18K ohms used in some testing.)
//!  The largest part of this file is the setup() functions used for each hal. 
//!  These make the application code common.

//!  This examples has been tested to run with both DHT11 and DHT22 on bluepill (stm32f103), 
//!  blackpill-stm32f401, and  blackpill-stm32f411. As of Feb. 2021 there are known
//!  runtime error on
//!  discovery-stm32f303 reads both DHT11 and DHT22  once but then fails at delay.
//!  discovery-stm32l100 gives Error Timeout with both DHT11 and DHT22. 
//!
//!  In linux with environment variables as below the example can be compiled, loaded and 
//!  run using gdb and openocd
//!  	openocd -f interface/$INTERFACE.cfg -f target/$PROC.cfg 
//!  In another window do
//!  	cargo  run --target $TARGET --features $HAL,$MCU    --example dht-multi-hals --release
//!  or if a DHT22 sensor is used
//!  	cargo  run --target $TARGET --features $HAL,$MCU,dht22 --example dht-multi-hals --release
//!  
//!  To only compile use
//!  	cargo  build --target $TARGET --features $HAL,$MCU --example dht-multi-hals [ --release ]
//!  or 
//!  	cargo  build --target $TARGET --features $HAL,$MCU,dht22 --example dht-multi-hals  [ --release ]
//!
//!  If --release is omitted then some MCUs do not have  sufficient memory an loading results in
//!       '.rodata will not fit in region FLASH ' 
//!
//!  However, even with sufficient memory the code without --release is slower and often results in an
//!  'Error Timeout' so it is recommended to use --release when attempting to run the code.
//!
//!                cargo run  environment variables                        openocd        test board and processor
//!    _____________________________________________________________     _____________   ___________________________
//!    export HAL=stm32f0xx MCU=stm32f042   TARGET=thumbv6m-none-eabi	 PROC=stm32f0x  # none-stm32f042      Cortex-M0
//!    export HAL=stm32f0xx MCU=stm32f030xc TARGET=thumbv6m-none-eabi	 PROC=stm32f0x  # none-stm32f030      Cortex-M0
//!    export HAL=stm32f1xx MCU=stm32f103   TARGET=thumbv7m-none-eabi	 PROC=stm32f1x  # bluepill	      Cortex-M3
//!    export HAL=stm32f1xx MCU=stm32f100   TARGET=thumbv7m-none-eabi	 PROC=stm32f1x  # none-stm32f100      Cortex-M3
//!    export HAL=stm32f1xx MCU=stm32f101   TARGET=thumbv7m-none-eabi	 PROC=stm32f1x  # none-stm32f101      Cortex-M3
//!    export HAL=stm32f3xx MCU=stm32f303xc TARGET=thumbv7em-none-eabihf PROC=stm32f3x  # discovery-stm32f303 Cortex-M3
//!    export HAL=stm32f4xx MCU=stm32f401   TARGET=thumbv7em-none-eabihf PROC=stm32f4x  # blackpill-stm32f401 Cortex-M4
//!    export HAL=stm32f4xx MCU=stm32f411   TARGET=thumbv7em-none-eabihf PROC=stm32f4x  # blackpill-stm32f411 Cortex-M4
//!    export HAL=stm32f4xx MCU=stm32f411   TARGET=thumbv7em-none-eabihf PROC=stm32f4x  # nucleo-64	      Cortex-M4
//!    hal NOT compiling as of (Feb 2021) export HAL=stm32f7xx MCU=stm32f722 TARGET=thumbv7em-none-eabihf #none-stm32f722 Cortex-M7
//!    export HAL=stm32h7xx MCU=stm32h742   TARGET=thumbv7em-none-eabihf                # none-stm32h742      Cortex-M7
//!    export HAL=stm32l0xx MCU=stm32l0x2   TARGET=thumbv6m-none-eabi	 PROC=stm32l1   # none-stm32l0x2      Cortex-M0
//!    export HAL=stm32l1xx MCU=stm32l100   TARGET=thumbv7m-none-eabi	 PROC=stm32l1   # discovery-stm32l100 Cortex-M3
//!    export HAL=stm32l1xx MCU=stm32l151   TARGET=thumbv7m-none-eabi	 PROC=stm32l1   # heltec-lora-node151 Cortex-M3
//!    NOT compiling as of (Feb 2021) export HAL=stm32l4xx MCU=stm32l4x2   TARGET=thumbv7em-none-eabi # none-stm32l4x1      Cortex-M4
//!  
//!  Depending on the MCU connection to the computer, in the  openocd command use
//!    export INTERFACE=stlink-v2  
//!    export INTERFACE=stlink-v2-1  

//! A version of this example is reported at https://pdgilbert.github.io/eg_stm_hal/. 
//! The results reported there use current git versions of the hals, whereas 
//! the testing reported above uses release versions of the hals (as of Feb 2021).

#![deny(unsafe_code)]
#![no_main]
#![no_std]


#[cfg(debug_assertions)]
extern crate panic_semihosting;

#[cfg(not(debug_assertions))]
extern crate panic_halt;

//use cortex_m::asm;  //for breakpoint
//asm::bkpt();

//use cortex_m;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

//https://github.com/michaelbeaumont/dht-sensor
use dht_sensor::*;

#[cfg(feature = "dht22")]
use dht_sensor::dht22::Reading;

#[cfg(not(feature = "dht22"))]
use dht_sensor::dht11::Reading;

//use crate::hal::{delay, gpio, prelude::*, stm32};

use embedded_hal::blocking::delay::{DelayMs,};

use embedded_hal::digital::v2::OutputPin;  // for  set_high().ok()

// setup() does all  hal/MCU specific setup and returns generic hal device for use in main code.

// See dht-sensor git discussion in issues #1  and #2
//https://github.com/michaelbeaumont/dht-sensor/issues/1
//Regarding pulling the pin high to avoid confusing the sensor when initializing.
//Also more in comments in dht-sensor crate file src/lib.rs

#[cfg(feature = "stm32f0xx")]
use stm32f0xx_hal::{prelude::*, 
                    pac::{Peripherals, CorePeripherals}, 
    	            delay::Delay,
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		    };

    // open_drain_output is really input and output

    #[cfg(feature = "stm32f0xx")]
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
      
       let cp      = CorePeripherals::take().unwrap();
       let mut p   = Peripherals::take().unwrap();
       let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);
      
       let gpioa  = p.GPIOA.split(&mut rcc);

       let mut pa8 = cortex_m::interrupt::free(move |cs| 
                   gpioa.pa8.into_open_drain_output(cs) );

       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 

       let mut delay = Delay::new(cp.SYST, &rcc);

       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);
      
       (pa8, delay)                 //DHT data will be on A8
       }

#[cfg(feature = "stm32f1xx")]
use stm32f1xx_hal::{prelude::*, 
                    pac::{Peripherals, CorePeripherals}, 
    	            delay::Delay,
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		    };

    // open_drain_output is really input and output

    #[cfg(feature = "stm32f1xx")]
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
      
       let cp = CorePeripherals::take().unwrap();
       let  p = Peripherals::take().unwrap();

       let mut rcc = p.RCC.constrain();
       let clocks = rcc.cfgr.freeze(&mut p.FLASH.constrain().acr);
       
       // delay is used by `dht-sensor` to wait for signals
       let mut delay = Delay::new(cp.SYST, clocks);   //SysTick: System Timer

       let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
       let mut pa8   = gpioa.pa8.into_open_drain_output(&mut gpioa.crh); 
       //let mut pa8 = cortex_m::interrupt::free(|cs| pa8.into_open_drain_output(cs));

       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 
 
       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);
      
       (pa8, delay)                //DHT data will be on A8
       }


#[cfg(feature = "stm32f3xx")]
use stm32f3xx_hal::{prelude::*, 
                    stm32::{Peripherals, CorePeripherals}, 
		    delay::Delay ,
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		    };

    #[cfg(feature = "stm32f3xx")]
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
       
       let cp = CorePeripherals::take().unwrap();
       let  p = Peripherals::take().unwrap();

       let mut rcc   = p.RCC.constrain();
       let clocks    = rcc.cfgr.freeze(&mut p.FLASH.constrain().acr);
       let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
       let mut pa8   = gpioa.pa8.into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);

       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 
       
       // delay is used by `dht-sensor` to wait for signals
       let mut delay = Delay::new(cp.SYST, clocks);   //SysTick: System Timer

       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);
       
       (pa8, delay)                  //DHT data will be on A8
       }


#[cfg(feature = "stm32f4xx")]
use stm32f4xx_hal::{prelude::*, 
                    stm32::{Peripherals, CorePeripherals},   //pac in newer releases
		    delay::Delay, 
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		    };

    #[cfg(feature = "stm32f4xx")]           // Use HSE oscillator
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
       
       let cp = CorePeripherals::take().unwrap();
       let  p = Peripherals::take().unwrap();
       let rcc = p.RCC.constrain();

       //let clocks =  p.RCC.constrain().cfgr.freeze();
       // next gives panicked at 'assertion failed: !sysclk_on_pll || 
       //                  sysclk <= sysclk_max && sysclk >= sysclk_min'
       //let clocks = p.RCC.constrain().cfgr.use_hse(8.mhz()).sysclk(168.mhz()).freeze();
       let clocks = rcc.cfgr.hclk(48.mhz()).sysclk(48.mhz()).pclk1(24.mhz()).pclk2(24.mhz()).freeze();

       hprintln!("sysclk freq: {}", clocks.sysclk().0).unwrap();  
       let mut pa8 = p.GPIOA.split().pa8.into_open_drain_output();  

       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 

              
       // delay is used by `dht-sensor` to wait for signals
       let mut delay = Delay::new(cp.SYST, clocks);   //SysTick: System Timer


       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);

       (pa8, delay)                  //DHT data will be on A8
       }


#[cfg(feature = "stm32f7xx")]
use stm32f7xx_hal::{prelude::*, 
                    pac::{Peripherals, CorePeripherals}, 
		    delay::Delay, 
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		    };

    #[cfg(feature = "stm32f7xx")]           // Use HSE oscillator
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
       
       let cp = CorePeripherals::take().unwrap();
       let  p = Peripherals::take().unwrap();
       let clocks = p.RCC.constrain().cfgr.sysclk(216.mhz()).freeze();

       let mut pa8 = p.GPIOA.split().pa8.into_open_drain_output();  

       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 
              
       // delay is used by `dht-sensor` to wait for signals
       let mut delay = Delay::new(cp.SYST, clocks);   //SysTick: System Timer

       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);

       (pa8, delay)                 //DHT data will be on A8
       }


#[cfg(feature = "stm32h7xx")]
use stm32h7xx_hal::{prelude::*, 
                    pac::{Peripherals, CorePeripherals}, 
		    delay::Delay, 
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		    };

    #[cfg(feature = "stm32h7xx")]  
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
       
       let cp = CorePeripherals::take().unwrap();
       let  p     = Peripherals::take().unwrap();
       let pwr    = p.PWR.constrain();
       let vos    = pwr.freeze();
       let rcc    = p.RCC.constrain();
       let ccdr   = rcc.sys_ck(160.mhz()).freeze(vos, &p.SYSCFG);
       let clocks = ccdr.clocks;

       let mut pa8 = p.GPIOA.split(ccdr.peripheral.GPIOA).pa8.into_open_drain_output();  
       
       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 
     
       // delay is used by `dht-sensor` to wait for signals
       let mut delay = Delay::new(cp.SYST, clocks);   //SysTick: System Timer

       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);

       (pa8, delay)                   //DHT data will be on A8
       }


#[cfg(feature = "stm32l0xx")]
use stm32l0xx_hal::{prelude::*, 
                    pac::{Peripherals, CorePeripherals}, 
		    rcc,   // for ::Config but note name conflict with serial
		    delay::Delay, 
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		    };

    #[cfg(feature = "stm32l0xx")]      
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
       
       let cp  = CorePeripherals::take().unwrap();
       let  p      = Peripherals::take().unwrap();
       let mut rcc = p.RCC.freeze(rcc::Config::hsi16());

       //let clocks =  p.RCC.constrain().cfgr.freeze();
       // next gives panicked at 'assertion failed: !sysclk_on_pll || 
       //                  sysclk <= sysclk_max && sysclk >= sysclk_min'
       //let clocks = p.RCC.constrain().cfgr.use_hse(8.mhz()).sysclk(168.mhz()).freeze();
       let mut pa8  = p.GPIOA.split(&mut rcc).pa8.into_open_drain_output();  

       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 
              
       // delay is used by `dht-sensor` to wait for signals
       //let mut delay = Delay::new(cp.SYST, clocks);   //SysTick: System Timer
       let mut delay = cp.SYST.delay(rcc.clocks);

       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);

       (pa8, delay)                //DHT data will be on A8
       }


#[cfg(feature = "stm32l1xx")]
use stm32l1xx_hal::{prelude::*, 
                    stm32::{Peripherals, CorePeripherals}, 
		    rcc,   // for ::Config but note name conflict with next
		    delay::Delay ,
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		   };

    #[cfg(feature = "stm32l1xx")]   
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
       
       let cp  = CorePeripherals::take().unwrap();
       let  p  = Peripherals::take().unwrap();
       let rcc = p.RCC.freeze(rcc::Config::hsi());

       //let clocks = p.RCC.constrain().cfgr.use_hse(8.mhz()).sysclk(168.mhz()).freeze();
       let mut pa8 = p.GPIOA.split().pa8.into_open_drain_output();

       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 
           
       // delay is used by `dht-sensor` to wait for signals
       //let mut delay = Delay::new(cp.SYST, clocks);   //SysTick: System Timer
          
       let mut delay = cp.SYST.delay(rcc.clocks);

       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);
   
       (pa8,  delay)                  //DHT data will be on A8
       }


#[cfg(feature = "stm32l4xx")]
use stm32l4xx_hal::{prelude::*, 
                    pac::{Peripherals, CorePeripherals}, 
		    delay::Delay, 
		    gpio::{gpioa::PA8, OpenDrain,  Output, },
		    };
//#[cfg(feature = "stm32l4xx")]
//use embedded_hal::digital::v2::{InputPin, OutputPin};

    #[cfg(feature = "stm32l4xx")]        
    fn setup() -> (PA8<Output<OpenDrain>>,  Delay) {
       
       let cp = CorePeripherals::take().unwrap();
       let  p = Peripherals::take().unwrap();
       let mut flash = p.FLASH.constrain();
       let mut rcc = p.RCC.constrain();
       let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);
       let clocks = rcc.cfgr .sysclk(80.mhz()) .pclk1(80.mhz()) 
                             .pclk2(80.mhz()) .freeze(&mut flash.acr, &mut pwr);

       let gpioa   = p.GPIOA.split(&mut rcc.ahb2);
       let mut pa8 = gpioa.pa8.into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
       
       // Pulling the pin high to avoid confusing the sensor when initializing.
       pa8.set_high().ok(); 
       
       // delay is used by `dht-sensor` to wait for signals
       let mut delay = Delay::new(cp.SYST, clocks);   //SysTick: System Timer

       //  1 second delay (for DHT setup?) Wait on  sensor initialization?
       delay.delay_ms(1000_u16);

       (pa8, delay)                   //DHT data will be on A8
       }


// End of hal/MCU specific setup. Following should be generic code.


#[entry]
fn main() -> ! {

    let (mut dht_data, mut delay) = setup();   //dht_data is usually pa8 in setup functions
    
    hprintln!("Reading sensor...").unwrap();
    
    // single read before loop for debugging purposes
    //
    //let r = Reading::read(&mut delay, &mut dht_data);
    //match r {
    //	Ok(Reading {
    //	    temperature,
    //	    relative_humidity,
    //	}) => hprintln!("{} deg C, {}% RH", temperature, relative_humidity).unwrap(),
    //	Err(e) => hprintln!("Error {:?}", e).unwrap(),
    //}

    loop {
        match Reading::read(&mut delay, &mut dht_data) {
            Ok(Reading {
                temperature,
                relative_humidity,
            }) => hprintln!("{} deg C, {}% RH", temperature, relative_humidity).unwrap(),
            Err(e) => hprintln!("Error {:?}", e).unwrap(),
        }

	// (Delay at least 500ms before re-polling, 1 second or more advised)
	// Delay 5 seconds
        delay.delay_ms(5000_u16);
    }
}
