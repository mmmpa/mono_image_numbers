use bit_iterator::BitIterator;
use itertools::Itertools;

pub type ImageNumberSource<T> = (u8, u8, T);

pub struct ImageNumberGenerator<T: AsRef<[u8]>, C: ImageNumberContainer> {
    height: u8,
    n0: ImageNumberSource<T>,
    n1: ImageNumberSource<T>,
    n2: ImageNumberSource<T>,
    n3: ImageNumberSource<T>,
    n4: ImageNumberSource<T>,
    n5: ImageNumberSource<T>,
    n6: ImageNumberSource<T>,
    n7: ImageNumberSource<T>,
    n8: ImageNumberSource<T>,
    n9: ImageNumberSource<T>,
    period: ImageNumberSource<T>,
    container: C,
}

impl<T: AsRef<[u8]>, C: ImageNumberContainer> ImageNumberGenerator<T, C> {
    pub fn new(
        height: u8,
        n0: ImageNumberSource<T>,
        n1: ImageNumberSource<T>,
        n2: ImageNumberSource<T>,
        n3: ImageNumberSource<T>,
        n4: ImageNumberSource<T>,
        n5: ImageNumberSource<T>,
        n6: ImageNumberSource<T>,
        n7: ImageNumberSource<T>,
        n8: ImageNumberSource<T>,
        n9: ImageNumberSource<T>,
        period: ImageNumberSource<T>,
        container: C,
    ) -> Self {
        Self {
            height,
            n0,
            n1,
            n2,
            n3,
            n4,
            n5,
            n6,
            n7,
            n8,
            n9,
            period,
            container,
        }
    }

    fn img(&self, n: u8) -> &ImageNumberSource<T> {
        match n {
            0 => &self.n0,
            1 => &self.n1,
            2 => &self.n2,
            3 => &self.n3,
            4 => &self.n4,
            5 => &self.n5,
            6 => &self.n6,
            7 => &self.n7,
            8 => &self.n8,
            9 => &self.n9,
            255 => &self.period,
            _ => &self.n0, // unreachable
        }
    }

    fn w(&self, n: u8) -> u8 {
        self.img(n).0
    }

    pub fn split_into_each_digit(&self, n: usize) -> (usize, [(u8, u8); 16], usize, usize) {
        let mut nums = [(0, 0); 16];
        let mut l = 0;
        let mut now = n;
        let mut width = 0;
        while now > 0 {
            let n = (now % 10) as u8;
            let w = self.w(n);
            width += w;
            nums[l] = (n, w);
            l += 1;
            now /= 10
        }

        (l, nums, width as usize + (l - 1), self.height as usize)
    }

    fn pixels(&self, n: u8) -> &[u8] {
        self.img(n).2.as_ref()
    }

    pub fn update_container(&mut self, n: usize) -> (usize, usize) {
        let (l, v, canvas_w, canvas_h) = self.split_into_each_digit(n);

        let mut offset = 0;

        for (i, (n, w)) in v[0..l].iter().rev().enumerate() {
            let owned = self
                .pixels(*n)
                .iter()
                .map(|n| *n)
                .flat_map(|n| BitIterator::from(n))
                .take(*w as usize * canvas_h)
                .collect::<Vec<_>>();

            owned
                .chunks(*w as usize)
                .into_iter()
                .enumerate()
                .for_each(|(y, row)| {
                    row.into_iter().enumerate().for_each(|(step_x, b)| {
                        self.container.edit(y * canvas_w + offset + step_x, *b)
                    });
                });

            offset += *w as usize + 1;

            if i == l - 1 {
                break;
            }

            // 文字間のスペース分の古いピクセルを消す
            for y in 0..canvas_h {
                self.container.edit(y * canvas_w + offset - 1, false)
            }
        }

        (canvas_w, canvas_h)
    }
}

pub trait Provider {
    fn src(&self, n: u8) -> &[u8];
    fn w(&self, n: u8) -> u8;
}

pub trait ImageNumberContainer {
    fn edit(&mut self, index: usize, b: bool);
}

#[cfg(test)]
mod tests {
    use crate::{ImageNumberContainer, ImageNumberGenerator};
    use itertools::Itertools;

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

    fn print(canvas_w: usize, canvas_h: usize, data: &[bool]) {
        for row in &data.iter().take(canvas_w * canvas_h).chunks(canvas_w) {
            row.for_each(|b| print!("{}", if *b { "■" } else { "□" }));
            print!("\n");
        }
    }

    impl ImageNumberContainer for [bool; 300] {
        fn edit(&mut self, index: usize, b: bool) {
            self[index] = b;
        }
    }

    fn numbers() -> ImageNumberGenerator<[u8; 10], [bool; 300]> {
        ImageNumberGenerator::new(
            10,
            VEC_NUM_0,
            VEC_NUM_1,
            VEC_NUM_2,
            VEC_NUM_3,
            VEC_NUM_4,
            VEC_NUM_5,
            VEC_NUM_6,
            VEC_NUM_7,
            VEC_NUM_8,
            VEC_NUM_9,
            VEC_NUM_PERIOD,
            [false; 300],
        )
    }

    #[test]
    fn test() {
        let mut n = numbers();

        let (w, h) = n.update_container(11185);
        print(w, h, &n.container);

        let (w, h) = n.update_container(20);
        print(w, h, &n.container);
    }
}
