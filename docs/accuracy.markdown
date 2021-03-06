# Emulation accuracy

## Open questions

### What happens if the CPU accesses memory during OAM DMA?

Writes are ignored and reads return $FF?
However, this might vary depending on the address, because $FF46 (OAM DMA register) writes still have an effect.

### What is the exact cycle-by-cycle behaviour of OAM DMA?

The GPU is probably able to access the OAM memory while an OAM DMA is active, so OAM DMA cannot be always emulated using a single big memory copy operation.

### What are supported source addresses for OAM DMA? Some sources claim that 0xE0-0xF1 are supported.

### What happens if you try to do OAM DMA with an unsupported source address?

### What happens if there is an interrupt during OAM DMA? Is this even possible?

If we assume that $FFFF is not readable by the CPU during OAM DMA, this would mean interrupts are not even possible.

### Do joypad interrupts depend on the select bits P14-P15, or do we get an interrupt whenever any key is pressed regardless of select bit state?

## Answered questions

### Does BIT b, (HL) take 12 or 16 T-cycles?

12 T-cycles.

Blargg's instruction timing ROM confirms that BIT takes 12, and RES/SET take 16 T-cycles, which makes perfect sense.
Some opcode listings in the internet (e.g. GBCPUman.pdf) are wrong.

### What is the exact behaviour of DI?

On DMG/MGB, DI has an immediate effect. On CGB/GBA, DI has a delay like EI (I'm really, really, doubting this but this is what the test result says right now).

*See test: di_timing*

### What is the exact behaviour of EI?

EI has a delay of one machine cycle. EI simply sets an internal flag, which will have an effect after the next instruction is executed. If you rapidly switch between DI/EI right after another, the internal flag has no effect during the switching, and the last instruction wins.

So, assuming interrupts are disabled, and an interrupt has already been requested, this code will cause only one interrupt:

    ei
    di
    ei
    di
    ei
    nop ; <- interrupt is acknowledged between these two
    nop ; <- instructions

*See: tests/ei_timing, tests/rapid_di_ei*

### Is it possible to restore the bootrom by writing some value to $FF50?

No.

This was tested on a GBP (MGB-001) with the following test ROM, which attempts to write all possible values to $FF50:

      ld hl, $0000
      ld b, $00         ; value to be written to $FF50

    - ld a, b
      ld ($FF00+$50), a
      ld a, (HL)
      cp $31            ; DMG bootrom should have $31 at $0000
      jr z, +
      inc b             ; attempt next value
      jr nz, -          ; retry until overflow

    + nop

      ; if A is $FF and B is $00, test failed
      ; A should be $31
      ; B should contain the written value

      jp finish

### The joypad register (P1) has only 4 inputs (P10-P13). What happens if you enable both key select bits P14-P15 and press overlapping keys?

Both sets of keys are "merged" in the input bits P10-P13. So, if you have both key select bits enabled and press any combination of A and Right, you will see P10 go down (= "set"). Also, if you press A and Right, and then stop pressing Right, P10 should still be down because A is still being pressed.

### What is the initial state of the joypad register (P1)? Does the boot rom write to it?

The DMG/GBP boot rom doesn't write to the joypad register, and the initial value is 0xCF.
This means that key select bits P14-P15 (bits 4-5) are low (= "set").

If GBC is used with old Gameboy games, the boot rom writes and reads from P1, because old games support
palette switches with certain key combinations during boot. After booting, the value is 0xFF.
This means all bits are high (= "unset").

### Does writing to DIV ($FF04) reset both the internal and the visible register?

*Answer:* Yes

DIV is incremented every 64 M-cycles, so there is an internal counter that counts to 64. If we write any value to the DIV register, it is reset to 0, but we don't know if the internal counter is also reset.

Consider the case where at time M=0 we reset the counter, and at time M=1 the DIV register would have incremented if we didn't do the reset. Do we see the DIV increment at time M=1 or M=64?

A test ROM confirmed that increment happens at M=64, so the internal counter is also reset.

*See: tests/div_timing*

### How many cycles does OAM DMA take?

OAM DMA takes 162 M-cycles. The following test returns $15 in counter register C:

      start_oam_dma
      nops 6
    - inc c
      ld a, (hl)
      cp $01
      jr nz, -

If we add one extra nop (= 7 nops in total), we get $14. In the 6 nops case, there are 19 ld a,(hl) calls which don't see data, and one call which sees the data. The total cycle count at the last failing call is 6 + 19 * (1 + 2 + 2 + 3) + (1 + 2) = 161 cycles. So, waiting for 161 cycles is not enough to see the DMA end. Adding one NOP makes the ld a, (hl) see memory normally, so therefore the total cycle length of OAM DMA is 162 cycles.

We are copying 40 x 32 bits = 160 bytes, so most likely we have one cycle per byte, and the extra 2 are startup/teardown cycles...

*See: tests/oam_dma_timing*

### What happens if another OAM DMA is requested while one is already active?

A new OAM DMA is started, so the entire process starts all over again.

*See: tests/oam_dma_restart*

### What is the exact timing of PUSH rr?

PUSH has an extra internal delay, which causes it to use 4 M-cycles (vs 3 cycles POP rr):

    M = 0: instruction decoding
    M = 1: internal delay
    M = 2: memory access for high byte
    M = 3: memory access for low byte

*See: tests/push_timing*

### What is the exact timing of CPU servicing an interrupt?

5 M-cycles in total involving internal delays and a PC push:

    M = 0: internal delay
    M = 1: internal delay
    M = 2: internal delay
    M = 3: PC push: memory access for high byte
    M = 4: PC push: memory access for low byte

*See: tests/intr_timing, tests/intr_timing2*

### What is the exact timing of LD HL, SP+e?

LD HL, SP+e has an extra internal delay after decoding and reading of e:

    M = 0: instruction decoding
    M = 1: memory access for e
    M = 2: internal delay

*See: tests/ld_hl_sp_e_timing*

### What is the exact timing of ADD SP, e?

ADD SP, e has two extra internal delays after decoding and reading of e:

    M = 0: instruction decoding
    M = 1: memory access for e
    M = 2: internal delay
    M = 3: internal delay

*See: tests/add_sp_e_timing*

### What is the exact timing of RST?

RST has an extra internal delay before the PC push:

    M = 0: instruction decoding
    M = 1: internal delay
    M = 2: PC push: memory access for high byte
    M = 3: PC push: memory access for low byte

*See: tests/rst_timing*

### What is the exact timing of CALL/JP/JR (not JP HL!)?

JP nn has an extra internal delay:

    M = 0: instruction decoding
    M = 1: nn read: memory access for low byte
    M = 2: nn read: memory access for high byte
    ; cc matches or unconditional
    M = 3: internal delay

JR n has an extra internal delay:

    M = 0: instruction decoding
    M = 1: n read: memory access
    ; cc matches or unconditional
    M = 2: internal delay

CALL has an extra internal delay before the PC push:

    M = 0: instruction decoding
    M = 1: nn read: memory access for low byte
    M = 2: nn read: memory access for high byte
    ; cc matches or unconditional
    M = 3: internal delay
    M = 4: PC push: memory access for high byte
    M = 5: PC push: memory access for low byte

*See: tests/call\_timing, tests/call\_timing2, tests/call\_cc\_timing, tests/jp\_timing, tests/jp\_cc\_timing*

### What does MBC1 do if you request a ROM bank number higher than what the cartridge supports?

The ROM bank numbers wrap around. Note that the bank 0 -> bank 1 quirk also appears with wrapped values! So, assuming a 32-bank cartridge:

* Requesting bank 0 gets you bank 1
* Requesting bank 1 gets you bank 1
* Requesting bank 32 wraps around to 0, which gets you bank 1
* Requesting bank 33 wraps around to 1, which gets you bank 1

This was verified using Arduino Uno, insidegadgets.com Cart Reader v1.2, and Wario Land cartridge (MBC1, 4Mbit ROM, 64Kbit RAM).

### On DMG, sprite flags in the OAM area have unused bits. Are they usable or do they always return 0 or 1?

They are unused but writable and readable normally.

*See: tests/oam\_bits*

### How does sprite priority work?

In my opinion it's easiest to think about two order: sprite priority order and drawing order.

Sprite priority order is based on the sprite position in the OAM. Sprites with lower position have higher priority.
Gameboy has a limit of 10 sprites per scanline, so when drawing a scanline, you simply select the first 10 visible sprites from the OAM data.

Drawing order depends on both OAM position and X coordinate. Sprites with small X coordinates have priority over sprites with large X coordinates.
So, in practice you draw the sprites in descending X order. If the X coordinate is the same for some sprites, you use the OAM position order (low position is drawn last).

### Some instructions take more cycles than just the memory accesses. At which point in the instruction execution do these extra cycles occur?

These instructions have just internal delays and no memory accesses, so the
timing does not matter as it is not observable:

* LD SP, HL
* ADD HL, rr
* INC rr
* DEC rr

These instructions involve writing a 16-bit register, which could explain the timing.

### We know that EI has a delayed effect. What happens if you do EI + HALT?

*Question proposed by Ricki Brown.*

If we execute EI right before HALT, tests show that the behaviour of HALT is the same as in a normal IME=1 case.
We don't know whether IME=0 or IME=1 in the hardware when the CPU is sleeping, but it doesn't actually matter!

If IME=1 when we enter HALT, it will wake up the CPU and service an interrupt as usual.

If IME=0 when we enter HALT, it will wake up the CPU, and execution continues to the next instruction.
At this point EI has an effect, IME becomes 1, and since interrupts are serviced before decoding, we end up servicing an interrupt anyway!

*See test: halt\_ime0\_ei*
