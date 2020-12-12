// Flash programming algorithm implementation


pub struct Flash {
    flash: stm32f4xx_hal::stm32::FLASH
}

const KEY1: u32= 0x45670123;
const KEY2: u32= 0xCDEF89AB;

impl Flash {
    pub fn new(flash: stm32f4xx_hal::stm32::FLASH) -> Self {
        Flash { flash }
    }

    fn unlock_cr(&self) {
        self.flash.keyr.write(|w| unsafe { w.bits(KEY1) });
        self.flash.keyr.write(|w| unsafe { w.bits(KEY2) });
    }    

    pub fn erase_sector(&self, sector: u8) {
        // Wait for flash to be ready
        while self.flash.sr.read().bsy().bit_is_set() {}
    
        self.unlock_cr();
    
        self.flash.cr.write(|w| unsafe {
            w.psize().bits(2) // Parallelism x32 (good for 3V power supply)
             .snb().bits(sector) // Sector number
             .ser().set_bit() // Sector erase
             .strt().set_bit() // Start!
        });
    
        // Wait for flash to be ready
        while self.flash.sr.read().bsy().bit_is_set() {}
    }

    pub fn program(&self, address: u32, data: &[u8]) {
        // Wait for flash to be ready
        while self.flash.sr.read().bsy().bit_is_set() {}

        self.unlock_cr();

        self.flash.cr.write(|w| unsafe {
            w.psize().bits(2) // Parallelism x32 (good for 3V power supply)
             .pg().set_bit()  // Set program bit
        });

        for i in 0..(data.len()/4) {
            let w: u32 = data[4*i] as u32 +
                         ((data[(4*i) + 1] as u32) << 8) +
                         ((data[(4*i) + 2] as u32) << 16) +
                         ((data[(4*i) + 3] as u32) << 24);
            
            let ptr = (address + (i as u32 *4)) as *mut u32;
            unsafe { *ptr = w; }

            // Wait for flash to be ready
            while self.flash.sr.read().bsy().bit_is_set() {}
        }
    }
}