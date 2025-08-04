use rand::{self, Rng};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const BG_WIDTH: usize = 256;
const BG_HEIGHT: usize = 256;

const TILEMAP_WIDTH: usize = 32;
const TILE_WIDTH: usize = 8;
const TILE_HEIGHT: usize = 8;

#[derive(Default)]
pub struct PPU {
    pub lcdc: u8,   // LCD control register
    pub scx: u8,    // Scroll X
    pub scy: u8,    // Scroll Y
    pub ly: u8,     // LCDC current Y position
    pub lyc: u8,    // LY compare
    pub wy: u8,     // Window Y position
    pub bgp: u8,    // Background palette (non-color mode)
    pub obp0: u8,   // Object palette 0 (non-color mode)
    pub obp1: u8,   // Object palette 1 (non-color mode)
    pub bgpi: u8,   // Background palette index (color mode)
    pub obpi: u8,   // Object palette index (color mode)

    stat: u8,       // LCDC status register
    bgpd: Vec<u8>,  // Background palette data (color mode)
    obpd: Vec<u8>,  // Object palette data (color mode)
}

// OAM sprite data
struct Sprite { y: u8, x: u8, tile: u8, attrs: u8 }

impl PPU {
    pub fn new() -> Self {
        let mut ppu = Self::default();
        // Background palette is initialized to all white on startup, but object palette is left as
        // random junk.
        for _ in 0..64 {
            ppu.bgpd.push(0xff);
            ppu.obpd.push(rand::rng().random());
        }
        ppu
    }

    // Get the color index of pixel (x,y) of the given tile. If select is false, use "0x8000"
    // addressing into VRAM tile data, and if select is true, use "0x8800" addressing.
    fn get_tile_pixel_color(tile: u8, x: usize, y: usize, vram: &Vec<u8>, select: bool) -> u8 {
        assert!(x < 8);
        assert!(y < 8);
        let index = if select {
            if tile < 128 { tile as usize + 256 } else { tile as usize + 128 }
        } else {
            tile as usize
        };
        let (byte0, byte1) = (vram[index*16 + y*2], vram[index*16 + y*2 + 1]);
        let (bit0, bit1) = ((byte0 >> (7-x)) & 0x1, (byte1 >> (7-x)) & 0x1);
        bit0 | (bit1 << 1)
    }

    fn put_color(pixel: &mut [u8], color: u8, palette: u8) {
        let rgba = match (palette >> (color*2)) & 0x3 {
            0b00 => [0x9b, 0xbc, 0x0f, 0xff],
            0b01 => [0x8b, 0xac, 0x0f, 0xff],
            0b10 => [0x30, 0x62, 0x30, 0xff],
            0b11 => [0x0f, 0x38, 0x0f, 0xff],
            _ => unreachable!()
        };
        pixel.copy_from_slice(&rgba);
    }

    pub fn draw_scanline(&mut self, framebuf: &mut [u8], vram: &Vec<u8>, oam: &Vec<u8>) {
        let y = self.ly as usize;
        let scanline_start = y * SCREEN_WIDTH * 4;
        let scanline = &mut framebuf[scanline_start..scanline_start+SCREEN_WIDTH*4];

        for (x, pixel) in scanline.chunks_exact_mut(4).enumerate() {
            let tilemap_x = ((x + self.scx as usize) % BG_WIDTH) / TILE_WIDTH;
            let tilemap_y = ((y + self.scy as usize) % BG_HEIGHT) / TILE_HEIGHT;

            let tilemap_index = tilemap_y * TILEMAP_WIDTH + tilemap_x;
            let tile = vram[0x1800 + tilemap_index];

            let tile_x = (x + self.scx as usize) % TILE_WIDTH;
            let tile_y = (y + self.scy as usize) % TILE_HEIGHT;
            let bg_color = Self::get_tile_pixel_color(tile, tile_x, tile_y, vram, false);

            let sprite = oam
                .chunks_exact(4)
                .map(|obj| Sprite { y: obj[0], x: obj[1], tile: obj[2], attrs: obj[3] })
                .filter(|sprite| {
                    let sx = sprite.x as isize - 8;
                    let sy = sprite.y as isize - 16;
                    (x as isize) >= sx && (x as isize) < sx + (TILE_WIDTH as isize)
                        && (y as isize) >= sy && (y as isize) < sy + (TILE_HEIGHT as isize)
                })
                .nth(0); // NOTE: CGB ranks priority by X rather than position in OAM

            if let Some(sprite) = sprite {
                let (spr_x, spr_y) = (x % TILE_WIDTH, y % TILE_HEIGHT);
                let spr_color = Self::get_tile_pixel_color(sprite.tile, spr_x, spr_y, vram, false);

                if spr_color == 0 {
                    Self::put_color(pixel, bg_color, self.bgp);
                } else if sprite.attrs & 0x10 != 0 {
                    Self::put_color(pixel, spr_color, self.obp1);
                } else {
                    Self::put_color(pixel, spr_color, self.obp0);
                }
            } else {
                Self::put_color(pixel, bg_color, self.bgp);
            }
        }

        self.ly = (self.ly + 1) % SCREEN_HEIGHT as u8;
    }

    pub fn get_stat(&self) -> u8 {
        self.stat
    }

    pub fn set_stat(&mut self, stat: u8) {
        self.stat = (self.stat & 0x7) | (stat & 0xf8);
    }

    pub fn get_bgpd(&self) -> u8 {
        self.bgpd[(self.bgpi & 0x3f) as usize]
    }

    pub fn set_bgpd(&mut self, value: u8) {
         self.bgpd[(self.bgpi & 0x3f) as usize] = value;  
    }

    pub fn get_obpd(&self) -> u8 {
        self.obpd[(self.obpi & 0x3f) as usize]
    }

    pub fn set_obpd(&mut self, value: u8) {
         self.obpd[(self.obpi & 0x3f) as usize] = value;
    }
}
