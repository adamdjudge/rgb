use rand::{self, Rng};

pub const SCANLINES: usize = 154;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const BG_WIDTH: usize = 256;
const BG_HEIGHT: usize = 256;

const TILEMAP_WIDTH: usize = 32;
const TILE_WIDTH: usize = 8;
const TILE_HEIGHT: usize = 8;

const LCDC_ON: u8 = 0x80;
const LCDC_WIN9C00: u8 = 0x40;
const LCDC_WINON: u8 = 0x20;
const LCDC_BG8000: u8 = 0x10;
const LCDC_BG9C00: u8 = 0x08;
const LCDC_OBJ16: u8 = 0x04;
const LCDC_OBJON: u8 = 0x02;
const LCDC_BGON: u8 = 0x01;

const STAT_ILYC: u8 = 0x40;
const STAT_IOAM: u8 = 0x20;
const STAT_IVBL: u8 = 0x10;
const STAT_IHBL: u8 = 0x08;
const STAT_LYC: u8 = 0x04;
const STAT_HBL: u8 = 0x00;
const STAT_VBL: u8 = 0x01;
const STAT_OAM: u8 = 0x02;
const STAT_LCD: u8 = 0x03;
const STAT_MODEMASK: u8 = 0x03;
const STAT_RWMASK: u8 = 0x78;

// OAM sprite data
struct Sprite { y: isize, x: isize, tile: u8, attrs: u8 }

impl Sprite {
    fn from(obj: &[u8]) -> Self {
        Self {
            y: obj[0] as isize - 16,
            x: obj[1] as isize - 8,
            tile: obj[2],
            attrs: obj[3],
        }
    }
}

#[derive(Default)]
pub struct PPU {
    pub lcdc: u8,   // LCD control register
    pub scx: u8,    // Scroll X
    pub scy: u8,    // Scroll Y
    pub ly: u8,     // LCDC current Y position
    pub lyc: u8,    // LY compare
    pub wy: u8,     // Window Y position
    pub wx: u8,     // Window X position minus 7
    pub bgp: u8,    // Background palette (non-color mode)
    pub obp0: u8,   // Object palette 0 (non-color mode)
    pub obp1: u8,   // Object palette 1 (non-color mode)
    pub bgpi: u8,   // Background palette index (color mode)
    pub obpi: u8,   // Object palette index (color mode)

    stat: u8,       // LCDC status register
    bgpd: Vec<u8>,  // Background palette data (color mode)
    obpd: Vec<u8>,  // Object palette data (color mode)
}

impl PPU {
    pub fn new() -> Self {
        let mut ppu = Self::default();
        // Background palette is initialized to all white on startup, but object palette is left as
        // random junk.
        for _ in 0..64 {
            ppu.bgpd.push(0xff);
            ppu.obpd.push(rand::rng().random());
        }
        ppu.lcdc = LCDC_ON | LCDC_BG8000 | LCDC_BGON;
        ppu.bgp = 0xFC;
        ppu.obp0 = 0xFF;
        ppu.obp1 = 0xFF;
        ppu
    }

    // Get the color index of pixel (x,y) of the given tile. If select is false, use "0x8000"
    // addressing into VRAM tile data, and if select is true, use "0x8800" addressing.
    fn get_tile_pixel_color(tile: u8, x: usize, y: usize, vram: &[u8], select: bool) -> u8 {
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

    pub fn draw_scanline(&mut self, framebuf: &mut [u8], vram: &[u8], oam: &[u8]) {
        let y = self.ly as usize;
        self.ly = (self.ly + 1) % SCANLINES as u8;
        if self.ly == self.lyc {
            self.stat |= STAT_LYC;
        } else {
            self.stat &= !STAT_LYC;
        }

        if y >= SCREEN_HEIGHT {
            return;
        }

        let mut sprites_this_line: Vec<Sprite> = oam
            .chunks_exact(4)
            .map(|obj| Sprite::from(obj))
            .filter(|s| (y as isize) >= s.y && (y as isize) < s.y + (TILE_HEIGHT as isize))
            .take(10)
            .collect();
        sprites_this_line.sort_by(|s1, s2| s1.x.cmp(&s2.x)); // NOTE: don't do this in CGB mode

        let scanline_start = y * SCREEN_WIDTH * 4;
        let scanline = &mut framebuf[scanline_start..scanline_start+SCREEN_WIDTH*4];

        for (x, pixel) in scanline.chunks_exact_mut(4).enumerate() {
            let tilemap_x = ((x + self.scx as usize) % BG_WIDTH) / TILE_WIDTH;
            let tilemap_y = ((y + self.scy as usize) % BG_HEIGHT) / TILE_HEIGHT;

            let tilemap_index = tilemap_y * TILEMAP_WIDTH + tilemap_x;
            let tile = vram[0x1800 + tilemap_index];

            let tile_x = (x + self.scx as usize) % TILE_WIDTH;
            let tile_y = (y + self.scy as usize) % TILE_HEIGHT;

            let bg_select = self.lcdc & LCDC_BG8000 == 0;
            let bg_color = Self::get_tile_pixel_color(tile, tile_x, tile_y, vram, bg_select);
            let bg_palette = if self.lcdc & LCDC_BGON == 0 { 0 } else { self.bgp };

            let sprite = sprites_this_line
                .iter()
                .find(|&s| (x as isize) >= s.x && (x as isize) < s.x + (TILE_WIDTH as isize));

            if let Some(sprite) = sprite {
                let spr_x = (x as isize - sprite.x) as usize;
                let spr_y = (y as isize - sprite.y) as usize;
                let spr_color = Self::get_tile_pixel_color(sprite.tile, spr_x, spr_y, vram, false);
                let spr_palette = if sprite.attrs & 0x10 == 0 { self.obp0 } else { self.obp1 };

                if spr_color == 0 || self.lcdc & LCDC_OBJON == 0 {
                    Self::put_color(pixel, bg_color, bg_palette);
                } else {
                    Self::put_color(pixel, spr_color, spr_palette);
                }
            } else {
                Self::put_color(pixel, bg_color, bg_palette);
            }
        }
    }

    pub fn get_stat(&self) -> u8 {
        self.stat
    }

    pub fn set_stat(&mut self, stat: u8) {
        self.stat = (self.stat & !STAT_RWMASK) | (stat & STAT_RWMASK);
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
