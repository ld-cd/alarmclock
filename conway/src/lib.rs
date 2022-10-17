#![cfg_attr(not(test), no_std)]

use core::num::Wrapping;

struct LFSR {
    state: u32,
}

impl LFSR {
    fn new(seed: u32) -> LFSR {
        if seed == 0 {
            LFSR { state: 1 }
        } else {
            LFSR { state: seed }
        }
    }
}

impl Iterator for LFSR {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let mut n = self.state;
        n ^= n << 13;
        n ^= n >> 17;
        n ^= n << 5;
        self.state = n;
        Some(self.state)
    }
}

#[derive(Debug)]
pub struct Conway<const W: usize, const H: usize> {
    pub counts: [[[Wrapping<u8>; H]; W]; 2],
    pub buf: usize,
}

impl<const W: usize, const H: usize> Conway<W, H> {
    pub fn new() -> Conway<W, H> {
        Conway {
            counts: [[[Wrapping(0); H]; W]; 2],
            buf: 0,
        }
    }
    fn clear(&mut self) {
        for x in 0..W {
            for y in 0..H {
                self.counts[0][x][y] = Wrapping(0);
            }
        }
        self.buf = 0;
    }

    pub fn randomize(&mut self, seed: u32) {
        self.clear();
        for x in 0..W {
            for (y, p) in LFSR::new(x as u32 + seed + 1).take(H).enumerate() {
                if (p as u8) & 0x80 != 0 {
                    self.counts[0][x][y] |= Wrapping(0x80 | 0x40);
                    self.delta(Wrapping(1), 0, x, y);
                }
            }
        }
        self.buf = 0;
    }

    pub fn step(&mut self) {
        const CM: Wrapping<u8> = Wrapping(0x1F_u8);
        const AM: Wrapping<u8> = Wrapping(0x80_u8);
        const UM: Wrapping<u8> = Wrapping(0x40_u8);

        let (nb, tb) = match self.buf {
            0 => (1, 0),
            _ => (0, 1),
        };

        for x in 0..W {
            for y in 0..H {
                let cpix = self.counts[tb][x][y];
                assert!(cpix & CM <= Wrapping(8));
                self.counts[nb][x][y] = match (cpix & CM, cpix & AM) {
                    (Wrapping(2), AM) => (cpix & CM) | AM,
                    (Wrapping(3), AM) => (cpix & CM) | AM,
                    (Wrapping(3), Wrapping(0)) => (cpix & CM) | AM | UM,
                    (_, AM) => (cpix & CM) | UM,
                    _ => (cpix & CM),
                }
            }
        }

        for x in 0..W {
            for y in 0..H {
                let pix = self.counts[nb][x][y];
                assert!(pix & CM <= Wrapping(8));
                match (pix & UM, pix & AM) {
                    (UM, Wrapping(0)) => self.delta(Wrapping(0xFF), nb, x, y),
                    (UM, AM) => self.delta(Wrapping(1), nb, x, y),
                    _ => (),
                }
            }
        }

        self.buf = nb;
    }

    fn delta(&mut self, delta: Wrapping<u8>, buf: usize, x: usize, y: usize) {
        self.counts[buf][x][(y + 1) % H] += delta;
        self.counts[buf][x][(y + H - 1) % H] += delta;
        self.counts[buf][(x + 1) % W][y] += delta;
        self.counts[buf][(x + W - 1) % W][y] += delta;
        self.counts[buf][(x + 1) % W][(y + 1) % H] += delta;
        self.counts[buf][(x + 1) % W][(y + H - 1) % H] += delta;
        self.counts[buf][(x + W - 1) % W][(y + 1) % H] += delta;
        self.counts[buf][(x + W - 1) % W][(y + H - 1) % H] += delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    fn conway_counts<const W: usize, const H: usize>(game: &Conway<W, H>, overwite: bool) {
        if overwite {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        }
        println!();
        for x in 0..W {
            for y in 0..H {
                print!("{}", game.counts[game.buf][x][y].0 % 10);
            }
            println!();
        }
    }

    fn conway_board<const W: usize, const H: usize>(game: &Conway<W, H>, overwite: bool) {
        if overwite {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        }
        println!();
        for x in 0..W {
            for y in 0..H {
                print!("{}", (game.counts[game.buf][x][y].0 & 0x80) >> 7);
            }
            println!();
        }
    }

    #[test]
    fn conway_step() {
        let mut game: Conway<128, 129> = Conway::new();
        game.randomize(8);
        for i in 0..512 {
            conway_board(&game, false);
            conway_counts(&game, false);
            println!("{}", i);
            game.step();
        }
    }
}
