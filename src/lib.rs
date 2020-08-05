use bit_iterator::BitIterator;

pub struct MonoImageNumberGenerator<P: SourceProvider, C: DataContainer> {
    height: u8,
    provider: P,
    container: C,
}

pub trait SourceProvider {
    fn pixels(&self, n: u8) -> &[u8];
    fn width(&self, n: u8) -> u8;
}

pub trait DataContainer {
    fn update(&mut self, index: usize, b: bool);
    fn data(&self) -> &[bool];
}

impl<P: SourceProvider, C: DataContainer> MonoImageNumberGenerator<P, C> {
    pub fn new(height: u8, provider: P, container: C) -> Self {
        Self {
            height,
            provider,
            container,
        }
    }

    pub fn each_digit(&self, n: usize) -> (usize, [(u8, u8); 16], usize, usize) {
        let mut nums = [(0, 0); 16];
        let mut l = 0;
        let mut now = n;
        let mut width = 0;
        while now > 0 {
            let n = (now % 10) as u8;
            let w = self.provider.width(n);
            width += w;
            nums[l] = (n, w);
            l += 1;
            now /= 10
        }

        (l, nums, width as usize + (l - 1), self.height as usize)
    }

    pub fn update_container(&mut self, n: usize) -> (usize, usize) {
        let (l, v, canvas_w, canvas_h) = self.each_digit(n);

        let mut offset = 0;

        for (i, (n, w)) in v[0..l].iter().rev().enumerate() {
            let owned = self
                .provider
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
                        self.container.update(y * canvas_w + offset + step_x, *b)
                    });
                });

            offset += *w as usize + 1;

            if i == l - 1 {
                break;
            }

            // 文字間のスペース分の古いピクセルを消す
            for y in 0..canvas_h {
                self.container.update(y * canvas_w + offset - 1, false)
            }
        }

        (canvas_w, canvas_h)
    }
}

#[cfg(test)]
mod tests {
    use crate::{DataContainer, MonoImageNumberGenerator, SourceProvider};
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

    struct ProviderClient;
    struct ContainerClient([bool; 300]);

    impl SourceProvider for ProviderClient {
        fn pixels(&self, n: u8) -> &[u8] {
            match n {
                0 => &VEC_NUM_0.2,
                1 => &VEC_NUM_1.2,
                2 => &VEC_NUM_2.2,
                3 => &VEC_NUM_3.2,
                4 => &VEC_NUM_4.2,
                5 => &VEC_NUM_5.2,
                6 => &VEC_NUM_6.2,
                7 => &VEC_NUM_7.2,
                8 => &VEC_NUM_8.2,
                9 => &VEC_NUM_9.2,
                255 => &VEC_NUM_PERIOD.2,
                _ => unreachable!(), // unreachable
            }
        }

        fn width(&self, n: u8) -> u8 {
            match n {
                0 => VEC_NUM_0.0,
                1 => VEC_NUM_1.0,
                2 => VEC_NUM_2.0,
                3 => VEC_NUM_3.0,
                4 => VEC_NUM_4.0,
                5 => VEC_NUM_5.0,
                6 => VEC_NUM_6.0,
                7 => VEC_NUM_7.0,
                8 => VEC_NUM_8.0,
                9 => VEC_NUM_9.0,
                255 => VEC_NUM_PERIOD.0,
                _ => unreachable!(), // unreachable
            }
        }
    }

    fn print(canvas_w: usize, canvas_h: usize, data: &[bool]) {
        for row in &data.iter().take(canvas_w * canvas_h).chunks(canvas_w) {
            row.for_each(|b| print!("{}", if *b { "■" } else { "□" }));
            print!("\n");
        }
    }

    impl DataContainer for ContainerClient {
        fn update(&mut self, index: usize, b: bool) {
            self.0[index] = b;
        }

        fn data(&self) -> &[bool] {
            &self.0
        }
    }

    fn numbers() -> MonoImageNumberGenerator<ProviderClient, ContainerClient> {
        MonoImageNumberGenerator::new(10, ProviderClient, ContainerClient([false; 300]))
    }

    #[test]
    fn test() {
        let mut n = numbers();

        let (w, h) = n.update_container(11185);
        print(w, h, n.container.data());

        let (w, h) = n.update_container(20);
        print(w, h, n.container.data());
    }
}
