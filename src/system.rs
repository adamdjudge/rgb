use rand::Rng;

use crate::ppu::*;

const ROM_SIZE: usize = 32 * 1024;
const VRAM_SIZE: usize = 8 * 1024;
const EXTRAM_SIZE: usize = 8 * 1024;
const WRAM_SIZE: usize = 8 * 1024;
const OAM_SIZE: usize = 160;
const HRAM_SIZE: usize = 127;

pub struct System {
    ppu: PPU,

    rom: Vec<u8>,
    vram: Vec<u8>,
    extram: Vec<u8>,
    wram: Vec<u8>,
    oam: Vec<u8>,
    hram: Vec<u8>,

    ie: u8,
}

impl System {
    pub fn new () -> Self {
        Self {
            ppu: PPU::new(),
            rom: vec![0; ROM_SIZE],
            vram: (0..VRAM_SIZE).map(|_| rand::rng().random()).collect(),
            extram: (0..EXTRAM_SIZE).map(|_| rand::rng().random()).collect(),
            wram: (0..WRAM_SIZE).map(|_| rand::rng().random()).collect(),
            oam: (0..OAM_SIZE).map(|_| rand::rng().random()).collect(),
            hram: (0..HRAM_SIZE).map(|_| rand::rng().random()).collect(),
            ie: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..0x8000 => self.rom[addr as usize],
            0x8000..0xA000 => self.vram[addr as usize - 0x8000],
            0xA000..0xC000 => self.extram[addr as usize - 0xA000],
            0xC000..0xE000 => self.wram[addr as usize - 0xC000],
            0xE000..0xFE00 => 0xFF,
            0xFE00..0xFEA0 => self.oam[addr as usize - 0xFE00],
            0xFEA0..0xFF00 => 0x00,
            0xFF40 => self.ppu.lcdc,
            0xFF41 => self.ppu.get_stat(),
            0xFF42 => self.ppu.scy,
            0xFF43 => self.ppu.scx,
            0xFF44 => self.ppu.ly,
            0xFF45 => self.ppu.lyc,
            0xFF47 => self.ppu.bgp,
            0xFF48 => self.ppu.obp0,
            0xFF49 => self.ppu.obp1,
            0xFF4A => self.ppu.wy,
            0xFF4B => self.ppu.wx,
            0xFF68 => self.ppu.bgpi,
            0xFF69 => self.ppu.get_bgpd(),
            0xFF6A => self.ppu.obpi,
            0xFF6B => self.ppu.get_obpd(),
            0xFF80..0xFFFF => self.hram[addr as usize - 0xFF80],
            0xFFFF => self.ie,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..0x8000 => self.rom[addr as usize] = data,
            0x8000..0xA000 => self.vram[addr as usize - 0x8000] = data,
            0xA000..0xC000 => self.extram[addr as usize - 0xA000] = data,
            0xC000..0xE000 => self.wram[addr as usize - 0xC000] = data,
            0xFE00..0xFEA0 => self.oam[addr as usize - 0xFE00] = data,
            0xFF40 => self.ppu.lcdc = data,
            0xFF41 => self.ppu.set_stat(data),
            0xFF42 => self.ppu.scy = data,
            0xFF43 => self.ppu.scx = data,
            0xFF44 => self.ppu.ly = data,
            0xFF45 => self.ppu.lyc = data,
            0xFF47 => self.ppu.bgp = data,
            0xFF48 => self.ppu.obp0 = data,
            0xFF49 => self.ppu.obp1 = data,
            0xFF4A => self.ppu.wy = data,
            0xFF4B => self.ppu.wx = data,
            0xFF68 => self.ppu.bgpi = data,
            0xFF69 => self.ppu.set_bgpd(data),
            0xFF6A => self.ppu.obpi = data,
            0xFF6B => self.ppu.set_obpd(data),
            0xFF80..0xFFFF => self.hram[addr as usize - 0xFF80] = data,
            0xFFFF => self.ie = data,
            _ => {},
        };
    }

    pub fn run_frame(&mut self, framebuf: &mut [u8]) {
        for _ in 0..SCANLINES {
            self.ppu.draw_scanline(framebuf, &mut self.vram, &mut self.oam);
        }
    }
}
