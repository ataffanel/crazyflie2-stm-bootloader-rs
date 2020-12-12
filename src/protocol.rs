use crate::syslink::SyslinkPacket;
use num_enum::TryFromPrimitive;
use crate::flash::Flash;

static FLASH_MAPPING: [u8; 6] = [4, 16, 1, 64, 7, 128];

static FLASH_SECTOR_ADDRESS: [u32; 12] = [
    0x08000000,
    0x08004000,
    0x08008000,
    0x0800C000,
    0x08010000,
    0x08020000,
    0x08040000,
    0x08060000,
    0x08080000,
    0x080A0000,
    0x080C0000,
    0x080E0000,
];

#[derive(defmt::Format, TryFromPrimitive)]
#[repr(u8)]
enum Command {
    GetInfo = 0x10,
    GetMapping = 0x12,
    LoadBuffer = 0x14,
    ReadBuffer = 0x15,
    WriteFlash = 0x18,
    FlashStatus = 0x19,
    ReadFlash = 0x1c,
}

pub fn handle_packet(packet: &mut SyslinkPacket,
                     buffers: &mut [[u8; 1024]; 10],
                     flash: &mut Flash) -> bool {

    if packet.length < 3 {
        return false;
    }

    if packet.buffer[1] != 0xff {
        return false;
    }

    let payload = &mut packet.buffer[2..];

    // let mut command_buffer = &mut packet.buffer[2..];

    if let Ok(command) = Command::try_from_primitive(payload[0]) {
        match command {
            Command::GetInfo => {
                // oageSize
                payload[1] = 0;
                payload[2] = 0x04;
                // nBuffPage
                payload[3] = 0x0A;
                payload[4] = 0x00;
                // nFlashPage
                payload[5] = 0x00;
                payload[6] = 0x04;
                // flashStart
                payload[7] = 0x10;
                payload[8] = 0x00;
                // Version
                payload[21] = 0x10;

                packet.length = 2 + 22;

                true
            },
            Command::LoadBuffer => {
                let page = payload[1] as usize + ((payload[2] as usize) << 8);
                let address = payload[3] as usize + ((payload[4] as usize) << 8);

                let lenght_to_copy = packet.length - 7;

                if (page < buffers.len()) && (address < buffers[page].len()) && (address + lenght_to_copy) <= buffers[page].len() {
                    buffers[page][address..address+lenght_to_copy].copy_from_slice(&payload[5..5+lenght_to_copy]);
                } else {
                    defmt::error!("Invalid page write received. Address: {:?}, Length: {:?}", address, lenght_to_copy);
                }

                false
            },
            Command::WriteFlash => {
                let buffer_page = payload[1] as usize + ((payload[2] as usize) << 8);
                let flash_page = payload[3] as usize + ((payload[4] as usize) << 8);
                let n_pages = payload[5] as usize + ((payload[6] as usize) << 8);



                defmt::info!("Flash request buffer {:?}, flash {:?}, nPages {:?}", buffer_page, flash_page, n_pages);

                for i in 0..n_pages {
                    let flash_address = 0x08000000 + ((i + flash_page) as u32 * 1024);

                    if let Ok(sector) = FLASH_SECTOR_ADDRESS.binary_search(&flash_address) {
                        defmt::info!("Erasing sector {:?}", sector);
                        flash.erase_sector(sector as u8);
                    }

                    defmt::info!("Flashing flash address {:?} with buffer page {:?}", flash_address, i);
                    flash.program(flash_address, &buffers[i]);
                }


                // TODO: Error should be handle
                payload[1] = 1;
                payload[2] = 0;

                packet.length = 2 + 1 + 2;

                true
            },
            Command::FlashStatus => {
                // TODO: Error should be handle
                payload[1] = 1;
                payload[2] = 0;

                packet.length = 2 + 1 + 2;

                true
            }
            Command::GetMapping => {
                payload[1..1+FLASH_MAPPING.len()].copy_from_slice(&FLASH_MAPPING);
                packet.length = 2 + 1 + FLASH_MAPPING.len();
                false
            },
            _ => false, // Uknown commands are ignored
        }
    } else {
        false
    }
}