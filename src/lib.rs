#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

use bit_iterator::BitIterator;

pub struct MonoImageNumbers<P: SourceProvider, C: DataContainer<M>, M: Copy> {
    height: u8,
    provider: P,
    // for splitting borrow
    // https://doc.rust-lang.org/nomicon/borrow-splitting.html
    container: Option<C>,
    mono: [M; 2],
}

pub trait SourceProvider {
    fn pixels(&self, n: char) -> &[u8];
    fn width(&self, n: char) -> u8;
}

pub trait DataContainer<M> {
    fn update(&mut self, index: usize, b: M);
    fn data(&self) -> &[M];
}

struct PassArgument {
    length: usize,
    char_and_width: [(char, u8); 16],
    canvas_width: usize,
}

impl<P: SourceProvider, C: DataContainer<M>, M: Copy> MonoImageNumbers<P, C, M> {
    pub fn new(height: u8, provider: P, container: C, mono: [M; 2]) -> Self {
        Self {
            height,
            provider,
            container: Some(container),
            mono,
        }
    }

    fn n_to_char(&self, n: u8) -> char {
        match n {
            0 => '0',
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            9 => '9',
            _ => unreachable!(), // unreachable
        }
    }

    fn each_digit(&self, n: isize) -> PassArgument {
        let mut char_and_width = [('-', 0); 16];
        let mut length = 0;
        let mut now = n;
        let mut canvas_width = 0;
        let mut minus = false;

        if now < 0 {
            minus = true;
            now *= -1;
        }

        if now == 0 {
            let w = self.provider.width('0');
            canvas_width += w as usize;
            char_and_width[length] = ('0', w);
            length += 1;

            return PassArgument {
                length,
                char_and_width,
                canvas_width,
            };
        }

        while now > 0 {
            let n = (now % 10) as u8;
            let c = self.n_to_char(n);
            let w = self.provider.width(c);
            canvas_width += w as usize;
            char_and_width[length] = (c, w);
            length += 1;
            now /= 10
        }

        if minus {
            let w = self.provider.width('-');
            canvas_width += w as usize;
            char_and_width[length] = ('-', w);
            length += 1;
        }

        // margin between chars
        canvas_width += length - 1;

        PassArgument {
            length,
            char_and_width,
            canvas_width,
        }
    }

    fn update_container(&mut self, each: PassArgument) -> (usize, usize) {
        let PassArgument {
            length,
            char_and_width,
            canvas_width: canvas_w,
        } = each;
        let canvas_h = self.height as usize;
        let mut offset = 0;
        let mut container = self.container.take().unwrap();

        for (i, (char, char_width)) in char_and_width[0..length].iter().rev().enumerate() {
            self.provider
                .pixels(*char)
                .iter()
                .map(|n| *n)
                .flat_map(|n| BitIterator::from(n))
                .take(*char_width as usize * canvas_h)
                .into_iter()
                .enumerate()
                .for_each(|(index, b)| {
                    let y = index / *char_width as usize;
                    let step_x = index % *char_width as usize;
                    container.update(
                        y * canvas_w + offset + step_x,
                        if b { self.mono[1] } else { self.mono[0] },
                    )
                });

            offset += *char_width as usize + 1;

            if i == length - 1 {
                break;
            }

            // 文字間のスペース分の古いピクセルを消す
            for y in 0..canvas_h {
                container.update(y * canvas_w + offset - 1, self.mono[0])
            }
        }

        self.container = Some(container);

        (canvas_w, canvas_h)
    }

    pub fn data(&self) -> &[M] {
        self.container.as_ref().unwrap().data()
    }

    pub fn update(&mut self, n: isize) -> (usize, usize) {
        self.update_container(self.each_digit(n))
    }

    pub fn update_f(&mut self, f: f32, level: usize) -> (usize, usize) {
        // at first, build not float data
        let n = (f * (10_i32.pow(level as u32) as f32) as f32) as isize;
        let PassArgument {
            length,
            mut char_and_width,
            canvas_width: canvas_w,
        } = self.each_digit(n);

        // then insert period
        let period_width = self.provider.width('.');

        for i in (level..length + 1).into_iter().rev() {
            char_and_width[i + 1] = char_and_width[i];
        }

        char_and_width[level] = ('.', period_width);

        self.update_container(PassArgument {
            length: length + 1,
            char_and_width,
            canvas_width: canvas_w + period_width as usize + 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{DataContainer, MonoImageNumbers, SourceProvider};
    use std::prelude::v1::*;

    const VEC_NUM_1: (u8, u8, [u8; 10]) = (3, 10, [0, 44, 151, 0, 0, 0, 0, 0, 0, 0]);
    const VEC_NUM_2: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 232, 136, 159, 0, 0, 0, 0, 0]);
    const VEC_NUM_3: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 232, 132, 193, 139, 128, 0, 0, 0]);
    const VEC_NUM_4: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 35, 42, 95, 16, 128, 0, 0, 0]);
    const VEC_NUM_5: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 244, 56, 33, 139, 128, 0, 0, 0]);
    const VEC_NUM_6: (u8, u8, [u8; 10]) = (5, 10, [34, 33, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_7: (u8, u8, [u8; 10]) = (5, 10, [0, 1, 248, 200, 68, 33, 0, 0, 0, 0]);
    const VEC_NUM_8: (u8, u8, [u8; 10]) = (5, 10, [116, 98, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_9: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 232, 197, 225, 17, 0, 0, 0, 0]);
    const VEC_NUM_0: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_PERIOD: (u8, u8, [u8; 10]) = (2, 10, [0, 15, 0, 0, 0, 0, 0, 0, 0, 0]);
    const VEC_NUM_MINUS: (u8, u8, [u8; 7]) = (4, 10, [0, 0, 240, 0, 0, 0, 0]);

    struct ProviderClient;
    struct ContainerClient([bool; 300]);

    impl SourceProvider for ProviderClient {
        fn pixels(&self, n: char) -> &[u8] {
            match n {
                '0' => &VEC_NUM_0.2,
                '1' => &VEC_NUM_1.2,
                '2' => &VEC_NUM_2.2,
                '3' => &VEC_NUM_3.2,
                '4' => &VEC_NUM_4.2,
                '5' => &VEC_NUM_5.2,
                '6' => &VEC_NUM_6.2,
                '7' => &VEC_NUM_7.2,
                '8' => &VEC_NUM_8.2,
                '9' => &VEC_NUM_9.2,
                '-' => &VEC_NUM_MINUS.2,
                '.' => &VEC_NUM_PERIOD.2,
                _ => unreachable!(), // unreachable
            }
        }

        fn width(&self, n: char) -> u8 {
            match n {
                '0' => VEC_NUM_0.0,
                '1' => VEC_NUM_1.0,
                '2' => VEC_NUM_2.0,
                '3' => VEC_NUM_3.0,
                '4' => VEC_NUM_4.0,
                '5' => VEC_NUM_5.0,
                '6' => VEC_NUM_6.0,
                '7' => VEC_NUM_7.0,
                '8' => VEC_NUM_8.0,
                '9' => VEC_NUM_9.0,
                '-' => VEC_NUM_MINUS.0,
                '.' => VEC_NUM_PERIOD.0,
                _ => unreachable!(), // unreachable
            }
        }
    }

    fn to_s(canvas_w: usize, canvas_h: usize, data: &[bool]) -> String {
        data.iter()
            .take(canvas_w * canvas_h)
            .fold(String::new(), |a, b| a + if *b { "⬛" } else { "⬜" })
    }

    impl DataContainer<bool> for ContainerClient {
        fn update(&mut self, index: usize, b: bool) {
            self.0[index] = b;
        }

        fn data(&self) -> &[bool] {
            &self.0
        }
    }

    fn numbers() -> MonoImageNumbers<ProviderClient, ContainerClient, bool> {
        MonoImageNumbers::new(
            10,
            ProviderClient,
            ContainerClient([false; 300]),
            [false, true],
        )
    }

    #[test]
    fn test_normal() {
        let mut n = numbers();

        let (w, h) = n.update(11185);
        assert_eq!(23, w);
        assert_eq!(10, h);
        assert_eq!(
            "\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬜⬜⬜\
                ⬜⬛⬜⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬛⬛⬛⬜⬜⬜⬛⬛⬛⬛\
                ⬛⬛⬜⬜⬛⬛⬜⬜⬛⬛⬜⬜⬛⬜⬜⬜⬛⬜⬜⬛⬜⬜⬜\
                ⬜⬛⬜⬜⬜⬛⬜⬜⬜⬛⬜⬜⬛⬜⬜⬜⬛⬜⬜⬛⬛⬛⬜\
                ⬜⬛⬜⬜⬜⬛⬜⬜⬜⬛⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬜⬜⬛\
                ⬛⬛⬛⬜⬛⬛⬛⬜⬛⬛⬛⬜⬜⬛⬛⬛⬜⬜⬜⬜⬜⬜⬛\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬛⬛⬜\
            ",
            to_s(w, h, n.data())
        );
    }

    #[test]
    fn test_update() {
        let mut n = numbers();

        n.update(11185);
        let (w, h) = n.update(20);
        assert_eq!(11, w);
        assert_eq!(10, h);
        assert_eq!(
            "\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬛⬛⬛⬜⬜⬜⬛⬛⬛⬜\
                ⬛⬜⬜⬜⬛⬜⬛⬜⬜⬜⬛\
                ⬜⬜⬜⬛⬜⬜⬛⬜⬜⬜⬛\
                ⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬛\
                ⬛⬛⬛⬛⬛⬜⬜⬛⬛⬛⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
            ",
            to_s(w, h, n.data())
        );
    }

    #[test]
    fn test_minus() {
        let mut n = numbers();

        let (w, h) = n.update(-119);
        assert_eq!(18, w);
        assert_eq!(10, h);
        assert_eq!(
            "\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬛⬛⬛⬜\
                ⬛⬛⬛⬛⬜⬛⬛⬜⬜⬛⬛⬜⬜⬛⬜⬜⬜⬛\
                ⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛⬜⬜⬛⬜⬜⬜⬛\
                ⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬛⬛⬛⬛\
                ⬜⬜⬜⬜⬜⬛⬛⬛⬜⬛⬛⬛⬜⬜⬜⬜⬜⬛\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬜⬜\
            ",
            to_s(w, h, n.data())
        );
    }

    #[test]
    fn test_float() {
        let mut n = numbers();

        let (w, h) = n.update_f(-19.1234, 2);
        assert_eq!(27, w);
        assert_eq!(10, h);
        assert_eq!(
            "\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛⬛⬛⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛⬛⬛⬜\
                ⬛⬛⬛⬛⬜⬛⬛⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬜⬛⬛⬜⬜⬛⬜⬜⬜⬛\
                ⬜⬜⬜⬜⬜⬜⬛⬜⬜⬛⬜⬜⬜⬛⬜⬜⬜⬜⬜⬛⬜⬜⬜⬜⬜⬛⬜\
                ⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬛⬛⬛⬛⬜⬛⬛⬜⬜⬛⬜⬜⬜⬜⬛⬜⬜\
                ⬜⬜⬜⬜⬜⬛⬛⬛⬜⬜⬜⬜⬜⬛⬜⬛⬛⬜⬛⬛⬛⬜⬛⬛⬛⬛⬛\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
                ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜\
            ",
            to_s(w, h, n.data())
        );
    }
}
