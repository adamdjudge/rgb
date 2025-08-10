include "hardware.inc"

def SHADOW_OAM equ $c000

; Interrupt vector for the vertical blanking interrupt
section "vblank_interrupt", ROM0[$40]
    jp _HRAM

; ROM entry point, which jumps past remaining header info to start code
section "header", ROM0[$100]
    jp start

;*******************************************************************************
; Variables in work RAM
;*******************************************************************************
def dx equ $c100
def dy equ $c101
def time equ $c102

;*******************************************************************************
; Startup initialization code
;*******************************************************************************
section "main", ROM0[$150]
start:
    di

    ; Disable audio
    ld a, 0
    ld [rNR52], a

    ; Wait for vblank before turning off the LCD
.wait_vblank:
    ld a, [rLY]
    cp a, 144
    jr c, .wait_vblank
    
    ; Turn off the LCD
    ld a, 0
    ld [rLCDC], a

    ; Copy OAM DMA routine into HIRAM
    ld hl, _HRAM
    ld bc, do_oam_dma
    ld d, do_oam_dma.end - do_oam_dma
    call memcpy

    ; Load sprite tiles into VRAM
    ld hl, _VRAM
    ld bc, sprite_tiles
    ld d, sprite_tiles.end - sprite_tiles
    call memcpy

    ; Load background tiles into VRAM
    ld hl, _VRAM9000
    ld bc, bg_tiles
    ld d, bg_tiles.end - bg_tiles
    call memcpy

    ; Initialize background map
    ld hl, _SCRN0
    ld bc, SCRN_VX_B * SCRN_VY_B
    call memclear

    ; Initialize shadow OAM
    ld hl, SHADOW_OAM
    ld bc, 4 * OAM_COUNT
    call memclear

    ; Setup sprite 0 in OAM as the ball
    ld a, 16
    ld [SHADOW_OAM + OAMA_Y], a
    ld a, 8
    ld [SHADOW_OAM + OAMA_X], a
    ld a, 1
    ld [SHADOW_OAM + OAMA_TILEID], a

    ; Initialize variables
    ld a, 1
    ld [dx], a
    ld [dy], a
    ld a, 0
    ld [time], a

    ; Set palettes
    ld a, %11100100
    ld [rBGP], a
    ld [rOBP0], a
    ld [rOBP1], a

    ; Turn on the LCD
    ld a, LCDCF_ON | LCDCF_BGON | LCDCF_OBJON
    ld [rLCDC], a

    ; Enable vblank interrupts
    ld a, IEF_VBLANK
    ld [rIE], a
    ei

;*******************************************************************************
; Main game loop, which halts until vblank to run each iteration
;*******************************************************************************
main_loop:
    halt

    ; Add dx to ball's x coordinate
.update_x:
    ld a, [dx]
    ld b, a
    ld a, [SHADOW_OAM + OAMA_X]
    add a, b
    ld [SHADOW_OAM + OAMA_X], a

    ; Check for collisions with left or right of screen to negate dx
    cp a, 8
    jr z, .negate_dx
    cp a, 160
    jr z, .negate_dx
    jr .update_y
.negate_dx:
    ld a, [dx]
    cpl
    inc a
    ld [dx], a

    ; Add dy to ball's y coordinate
.update_y:
    ld a, [dy]
    ld b, a
    ld a, [SHADOW_OAM + OAMA_Y]
    add a, b
    ld [SHADOW_OAM + OAMA_Y], a

    ; Check for collisions with top or bottom of screen to negate dy
    cp a, 16
    jr z, .negate_dy
    cp a, 152
    jr z, .negate_dy
    jr .check_scroll
.negate_dy:
    ld a, [dy]
    cpl
    inc a
    ld [dy], a

    ; If time variable is modulo 4, scroll the background
.check_scroll:
    ld a, [time]
    and a, $03
    jr nz, .inc_time
    ld a, [rSCX]
    inc a
    ld [rSCX], a

    ; Increment time variable
.inc_time:
    ld a, [time]
    inc a
    ld [time], a

    jr main_loop

;*******************************************************************************
; Helper function to copy memory from source, pointed to by bc, to destination,
; pointed to by hl, with length in d.
;*******************************************************************************
memcpy:
    ld a, [bc]
    ld [hl+], a
    inc bc
    dec d
    jr nz, memcpy
    ret

;*******************************************************************************
; Helper function to clear bytes in memory starting at hl with length bc.
;*******************************************************************************
memclear:
    ld a, 0
    ld [hl+], a
    dec bc
    ld a, b
    or a, c
    jr nz, memclear
    ret

;*******************************************************************************
; Routine for copying the shadow OAM into the real OAM via DMA. It's copied into
; HIRAM during startup, since code that uses DMA needs to be there.
;*******************************************************************************
do_oam_dma:
    ; Write high byte of shadow OAM address to the DMA register
    ld a, HIGH(SHADOW_OAM)
    ldh [rDMA], a

    ; Wait exactly 160 cycles for DMA transfer to finish
    ld a, 40
.loop
    dec a
    jr nz, .loop
    reti
.end

;*******************************************************************************
; Sprite and background tile data
;*******************************************************************************
section "tiles", ROM0
sprite_tiles:
    db $00, $00, $00, $00, $00, $00, $00, $00
    db $00, $00, $00, $00, $00, $00, $00, $00
    db $3C, $3C, $42, $7E, $8D, $F3, $85, $FB
    db $81, $FF, $81, $FF, $42, $7E, $3C, $3C
.end

bg_tiles:
    db $02, $00, $07, $00, $02, $00, $00, $00
    db $20, $00, $70, $00, $20, $00, $00, $00
.end
